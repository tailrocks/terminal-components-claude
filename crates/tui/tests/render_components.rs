//! The component digest matrix — `render::components::<component>::<state>`
//! (`COMPONENT_ARCHITECTURE.md` §16.3, Slice 2 acceptance condition 5).
//!
//! Twenty components × eight states × `{junie, paper}` × `{truecolor, mono}` ×
//! `{120×40, 40×10}` = 1280 checked-in digest lines in
//! `tests/baselines/components.txt`. The theme, colour level and size are part
//! of the baseline **key**, so one test function owns eight lines and a
//! regression names the exact cell of the matrix that moved.
//!
//! **File placement.** §16.3 puts this matrix in `tests/render.rs`. It is a
//! separate target because `tests/render.rs` is owned by the foundations work
//! package and this file by the components one; the test *paths*
//! (`render::components::…`) are what §16.3 and the acceptance condition name,
//! and they are identical either way. Merge the two targets at Slice 5 if the
//! split stops paying for itself.
//!
//! Every state is forced with `.state_override` (A11), so a digest is a pure
//! function of `(props, theme, colour, size)` — no focus, no pointer, no
//! frame counter. `empty` is the only state expressed as *content*: it is the
//! component with nothing to show.
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

use tui_next::{
    Action, ActionKey, Brand, Button, Checkbox, ChipBar, ChipBarState, Chord, ColorLevel, Dialog,
    DialogState, Empty, EmptyState, Field, Hint, HintBar, HintLayer, Id, ItemKey, KeyCode, KeyHint,
    List, ListState, Meter, ProgressBar, RadioGroup, RadioGroupState, Rect, RowUi, Select,
    SelectMode, SelectState, Spinner, StateFlags, Status, StatusBar, StatusItem, Tabs, TabsState,
    TextArea, TextAreaState, TextInput, TextInputState, Theme, Toggle, Ui, Variant,
};
use tui_next_testing::{Baseline, Scene};

const BASELINE: Baseline = Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/components.txt"
));

const BTN: Id = Id::root("render.button");
const INPUT: Id = Id::root("render.text_input");
const FIELD: Id = Id::root("render.field");
const LIST: Id = Id::root("render.list");
const TABS: Id = Id::root("render.tabs");
const DLG: Id = Id::root("render.dialog");
const TEXT_AREA: Id = Id::root("render.text_area");
const SELECT: Id = Id::root("render.select");
const RADIO_GROUP: Id = Id::root("render.radio_group");
const CHECKBOX: Id = Id::root("render.checkbox");
const TOGGLE: Id = Id::root("render.toggle");
const CHIP_BAR: Id = Id::root("render.chip_bar");
const STATUS_BAR: Id = Id::root("render.status_bar");
const HINT_BAR: Id = Id::root("render.hint_bar");
const KEY_HINT: Id = Id::root("render.key_hint");
const PROGRESS_BAR: Id = Id::root("render.progress_bar");
const SPINNER: Id = Id::root("render.spinner");
const METER: Id = Id::root("render.meter");
const EMPTY: Id = Id::root("render.empty");
const BRAND: Id = Id::root("render.brand");

/// `(label, meta)` rows for the list.
const ROWS: [(&str, &str); 6] = [
    ("Ada Lovelace", "analyst"),
    ("Grace Hopper", "rear admiral"),
    ("Alan Turing", "logician"),
    ("Edsger Dijkstra", "professor"),
    ("Barbara Liskov", "professor"),
    ("Ken Thompson", "engineer"),
];

const TAB_LABELS: [&str; 5] = ["General", "Mounts", "Roles", "Audit", "Advanced"];

const DIALOG_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "OK"),
];

/// The eight states the matrix renders.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum St {
    Default,
    Focused,
    Hovered,
    Pressed,
    Disabled,
    Selected,
    Editing,
    /// Not a state but the absence of content: no rows, no tabs, no value.
    Empty,
}

impl St {
    /// The flags `.state_override` forces; `Empty` forces nothing.
    const fn flags(self) -> StateFlags {
        match self {
            St::Default | St::Empty => StateFlags::empty(),
            St::Focused => StateFlags::FOCUSED.union(StateFlags::FOCUS_VISIBLE),
            St::Hovered => StateFlags::HOVERED,
            St::Pressed => StateFlags::PRESSED.union(StateFlags::FOCUSED),
            St::Disabled => StateFlags::DISABLED,
            St::Selected => StateFlags::SELECTED,
            St::Editing => StateFlags::EDITING,
        }
    }

    const fn is_empty(self) -> bool {
        matches!(self, St::Empty)
    }

    /// The `::<state>` half of the baseline key, matching `matrix!`'s
    /// `stringify!`d function names.
    const fn name(self) -> &'static str {
        match self {
            St::Default => "default",
            St::Focused => "focused",
            St::Hovered => "hovered",
            St::Pressed => "pressed",
            St::Disabled => "disabled",
            St::Selected => "selected",
            St::Editing => "editing",
            St::Empty => "empty",
        }
    }
}

/// The twenty components the matrix covers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Comp {
    Button,
    TextInput,
    Field,
    List,
    Tabs,
    Dialog,
    TextArea,
    Select,
    RadioGroup,
    Checkbox,
    Toggle,
    ChipBar,
    StatusBar,
    HintBar,
    KeyHint,
    ProgressBar,
    Spinner,
    Meter,
    Empty,
    Brand,
}

impl Comp {
    /// Every component the matrix registers, with the `render::components::`
    /// prefix's middle segment.
    ///
    /// Hand-written, and cross-checked against the baseline's own line count
    /// by `theme::readiness_states_are_digest_distinct`: a component added to
    /// `matrix!` but not here makes the baseline hold 64 more digest lines
    /// than this list accounts for, and that check fails.
    const ALL: [(&'static str, Comp); 20] = [
        ("button", Comp::Button),
        ("text_input", Comp::TextInput),
        ("field", Comp::Field),
        ("list", Comp::List),
        ("tabs", Comp::Tabs),
        ("dialog", Comp::Dialog),
        ("text_area", Comp::TextArea),
        ("select", Comp::Select),
        ("radio_group", Comp::RadioGroup),
        ("checkbox", Comp::Checkbox),
        ("toggle", Comp::Toggle),
        ("chip_bar", Comp::ChipBar),
        ("status_bar", Comp::StatusBar),
        ("hint_bar", Comp::HintBar),
        ("key_hint", Comp::KeyHint),
        ("progress_bar", Comp::ProgressBar),
        ("spinner", Comp::Spinner),
        ("meter", Comp::Meter),
        ("empty", Comp::Empty),
        ("brand", Comp::Brand),
    ];

    /// The [`Status`] this fixture hands `comp` in state `st`, or `None` when
    /// the fixture drives it with no status prop at all.
    ///
    /// **This is the single declaration of which components are
    /// status-driven.** The four `draw_*` functions that paint a status take
    /// it as an argument rather than calling `status_for` themselves, so the
    /// set `theme::readiness_states_are_digest_distinct` iterates cannot drift
    /// away from the set the fixture actually drives. The match is exhaustive
    /// with no `_` arm, so a twenty-first component has to be classified here
    /// before it compiles.
    ///
    /// `Empty` is deliberately `None`: it is driven by an [`EmptyState`]
    /// variant and re-derives its own `BUSY`/`LOADING`/`ERROR` flags, so it
    /// carries no `Status` prop for the property to be about.
    const fn status_prop(self, st: St) -> Option<Status> {
        match self {
            Comp::StatusBar | Comp::HintBar | Comp::ProgressBar | Comp::Meter => {
                Some(status_for(st))
            }
            Comp::Button
            | Comp::TextInput
            | Comp::Field
            | Comp::List
            | Comp::Tabs
            | Comp::Dialog
            | Comp::TextArea
            | Comp::Select
            | Comp::RadioGroup
            | Comp::Checkbox
            | Comp::Toggle
            | Comp::ChipBar
            | Comp::KeyHint
            | Comp::Spinner
            | Comp::Empty
            | Comp::Brand => None,
        }
    }
}

fn row_key(r: &(&str, &str)) -> ItemKey {
    ItemKey::text(r.0)
}

fn row_paint(r: &(&str, &str), u: &mut RowUi<'_>) {
    u.label(r.0);
    u.meta(r.1);
}

fn tab_paint(r: &&'static str, u: &mut RowUi<'_>) {
    u.label(r);
}

fn tab_key(r: &&'static str) -> ItemKey {
    ItemKey::text(r)
}

fn text_input(id: Id, st: St) -> TextInput<'static> {
    let mut t = TextInput::new(id).placeholder("Type a name");
    if !st.is_empty() {
        t = t.value("Ada Lovelace");
    }
    t.state_override(st.flags())
}

fn text_area(st: St) -> TextArea<'static> {
    let mut t = TextArea::new(TEXT_AREA, 4).placeholder("Type a note");
    if !st.is_empty() {
        t = t.value("Ada Lovelace\nanalyst");
    }
    t.state_override(st.flags())
}

/// The readiness the fixture reports for each matrix state. Reached only
/// through [`Comp::status_prop`], so the mapping and the set of components it
/// applies to are declared in one place.
const fn status_for(st: St) -> Status {
    match st {
        St::Pressed => Status::Busy,
        St::Editing => Status::Loading,
        St::Disabled => Status::Error,
        St::Default | St::Focused | St::Hovered | St::Selected | St::Empty => Status::Ready,
    }
}

fn empty_surface(st: St) -> EmptyState<'static> {
    match st {
        St::Empty => EmptyState::Empty {
            title: "",
            hint: None,
        },
        St::Editing => EmptyState::Loading { label: "Loading" },
        St::Disabled => EmptyState::Error {
            message: "Unable to load",
            detail: Some("Try again"),
        },
        St::Default | St::Focused | St::Hovered | St::Pressed | St::Selected => EmptyState::Empty {
            title: "No results",
            hint: Some("Try a different filter"),
        },
    }
}

const STATUS_LEFT: [StatusItem<'static>; 2] = [
    StatusItem::new("Workspace").strong(),
    StatusItem::new("main").key(ItemKey::num(1)),
];
const STATUS_CENTER: [StatusItem<'static>; 1] = [StatusItem::new("Ready")];
const STATUS_RIGHT: [StatusItem<'static>; 1] = [StatusItem::new("0 changes").key(ItemKey::num(2))];

/// The `(label, meta)` rows a collection renders in state `st`: `St::Empty` is
/// the absence of content, every other state gets the full set.
fn rows_for(st: St) -> &'static [(&'static str, &'static str)] {
    if st.is_empty() { &[] } else { &ROWS }
}

/// The tab labels for state `st`.
fn tabs_for(st: St) -> &'static [&'static str] {
    if st.is_empty() { &[] } else { &TAB_LABELS }
}

fn draw_button(st: St, ui: &mut Ui<'_>, area: Rect) {
    let label = if st.is_empty() { "" } else { "Run task" };
    Button::new(BTN, label)
        .variant(Variant::PRIMARY)
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_text_input(st: St, ui: &mut Ui<'_>, area: Rect) {
    text_input(INPUT, st).draw(ui, area, &TextInputState::default());
}

fn draw_field(st: St, ui: &mut Ui<'_>, area: Rect) {
    let label = if st.is_empty() { "" } else { "Name" };
    let mut f = Field::new(label, text_input(FIELD, st))
        .required(true)
        .state_override(st.flags());
    if !st.is_empty() {
        f = f.help("The person's display name.");
    }
    f.draw(ui, area, &TextInputState::default());
}

fn draw_list(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&(&str, &str)) -> ItemKey = row_key;
    let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
    List::new(LIST)
        .key(key)
        .row(row)
        .state_override(st.flags())
        .draw(ui, area, &ListState::default(), rows_for(st));
}

fn draw_tabs(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&&'static str) -> ItemKey = tab_key;
    let row: fn(&&'static str, &mut RowUi<'_>) = tab_paint;
    Tabs::new(TABS)
        .key(key)
        .row(row)
        .closable(true)
        .allow_new(true)
        .state_override(st.flags())
        .draw(ui, area, &TabsState::default(), tabs_for(st));
}

fn draw_dialog(st: St, ui: &mut Ui<'_>, area: Rect) {
    let d = if st.is_empty() {
        Dialog::new(DLG).body_rows(0)
    } else {
        Dialog::new(DLG)
            .title("Delete table")
            .description("This cannot be undone. Every row and every index goes with it.")
            .body_rows(0)
            .actions(&DIALOG_ACTIONS)
            .cancel(ActionKey::CANCEL)
    };
    d.state_override(st.flags())
        .draw(ui, area, &DialogState::default(), |_, _| {});
}

fn draw_text_area(st: St, ui: &mut Ui<'_>, area: Rect) {
    text_area(st).draw(ui, area, &TextAreaState::default());
}

fn draw_select(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&(&str, &str)) -> ItemKey = row_key;
    let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
    let mut state = SelectState::default();
    if !st.is_empty() {
        state.set_value(Some(ItemKey::text("Ada Lovelace")));
    }
    Select::new(SELECT)
        .key(key)
        .row(row)
        .placeholder("Choose a person")
        .popup_rows(5)
        .state_override(st.flags())
        .draw(ui, area, &state, rows_for(st));
}

fn draw_radio_group(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&(&str, &str)) -> ItemKey = row_key;
    let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
    RadioGroup::new(RADIO_GROUP)
        .key(key)
        .row(row)
        .value(ItemKey::text("Ada Lovelace"))
        .state_override(st.flags())
        .draw(ui, area, &RadioGroupState::default(), rows_for(st));
}

fn draw_checkbox(st: St, ui: &mut Ui<'_>, area: Rect) {
    Checkbox::new(CHECKBOX, "Accept terms")
        .checked(matches!(st, St::Selected))
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_toggle(st: St, ui: &mut Ui<'_>, area: Rect) {
    Toggle::new(TOGGLE, "Notifications")
        .on(matches!(st, St::Selected))
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_chip_bar(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&(&str, &str)) -> ItemKey = row_key;
    let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
    let mut state = ChipBarState::default();
    if matches!(st, St::Selected) {
        state.checked_mut().insert(ItemKey::text("Ada Lovelace"));
    }
    ChipBar::new(CHIP_BAR)
        .key(key)
        .row(row)
        .select_mode(SelectMode::Multi)
        .closable(true)
        .state_override(st.flags())
        .draw(ui, area, &state, rows_for(st));
}

fn draw_status_bar(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    let left: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_LEFT };
    let center: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_CENTER };
    let right: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_RIGHT };
    StatusBar::new(STATUS_BAR)
        .left(left)
        .center(center)
        .right(right)
        .status(status)
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_hint_bar(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    let layer = HintLayer {
        hints: if st.is_empty() {
            Vec::new()
        } else {
            vec![
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
            ]
        },
        badge: if st.is_empty() { None } else { Some("F1") },
        status: if st.is_empty() {
            None
        } else {
            Some(std::borrow::Cow::Borrowed("Ready"))
        },
        centered: false,
    };
    HintBar::new(HINT_BAR, &layer)
        .status(status)
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_key_hint(st: St, ui: &mut Ui<'_>, area: Rect) {
    KeyHint::new(KEY_HINT, Chord::key(KeyCode::Enter), "Open")
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_progress_bar(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    ProgressBar::new(PROGRESS_BAR)
        .label(if st.is_empty() { "" } else { "Uploading" })
        .ratio(if st.is_empty() { 0.0 } else { 0.65 })
        .status(status)
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_spinner(st: St, ui: &mut Ui<'_>, area: Rect) {
    Spinner::new(SPINNER)
        .label(if st.is_empty() { "" } else { "Working" })
        .frame(1)
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_meter(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    Meter::new(METER)
        .ratio(if st.is_empty() { 0.0 } else { 0.65 })
        .value(if st.is_empty() { "" } else { "65%" })
        .status(status)
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_empty(st: St, ui: &mut Ui<'_>, area: Rect) {
    Empty::new(EMPTY, empty_surface(st))
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_brand(st: St, ui: &mut Ui<'_>, area: Rect) {
    Brand::new(BRAND, if st.is_empty() { "" } else { "Junie" })
        .tagline(if st.is_empty() { "" } else { "Terminal tools" })
        .state_override(st.flags())
        .draw(ui, area);
}

/// Draw `comp` in state `st` into `area`.
///
/// Dispatch only: every arm is one `draw_*` above, so adding a component adds
/// a function and a line rather than growing one function without bound.
fn draw(comp: Comp, st: St, ui: &mut Ui<'_>, area: Rect) {
    // the four status-driven components receive their status as an argument,
    // so `Comp::status_prop` is the only place the fixture decides which
    // components report readiness
    let status = |c: Comp| {
        c.status_prop(st).unwrap_or_else(|| {
            panic!("{c:?} paints a status prop but Comp::status_prop returns None for {st:?}")
        })
    };
    match comp {
        Comp::Button => draw_button(st, ui, area),
        Comp::TextInput => draw_text_input(st, ui, area),
        Comp::Field => draw_field(st, ui, area),
        Comp::List => draw_list(st, ui, area),
        Comp::Tabs => draw_tabs(st, ui, area),
        Comp::Dialog => draw_dialog(st, ui, area),
        Comp::TextArea => draw_text_area(st, ui, area),
        Comp::Select => draw_select(st, ui, area),
        Comp::RadioGroup => draw_radio_group(st, ui, area),
        Comp::Checkbox => draw_checkbox(st, ui, area),
        Comp::Toggle => draw_toggle(st, ui, area),
        Comp::ChipBar => draw_chip_bar(st, ui, area),
        Comp::StatusBar => draw_status_bar(st, status(comp), ui, area),
        Comp::HintBar => draw_hint_bar(st, status(comp), ui, area),
        Comp::KeyHint => draw_key_hint(st, ui, area),
        Comp::ProgressBar => draw_progress_bar(st, status(comp), ui, area),
        Comp::Spinner => draw_spinner(st, ui, area),
        Comp::Meter => draw_meter(st, status(comp), ui, area),
        Comp::Empty => draw_empty(st, ui, area),
        Comp::Brand => draw_brand(st, ui, area),
    }
}

/// Render one matrix cell name across both themes, both colour levels and
/// both sizes, and compare every digest against the checked-in baseline.
fn run(name: &'static str, comp: Comp, st: St) {
    for theme in [Theme::junie(), Theme::paper()] {
        for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
            for (w, h) in [(120u16, 40u16), (40, 10)] {
                let mut scene = Scene::new(name, theme.clone(), color, w, h);
                scene.draw(|ui, area| draw(comp, st, ui, area));
                scene.assert_against(&BASELINE);
            }
        }
    }
}

macro_rules! matrix {
    ($comp:ident, $c:expr) => {
        mod $comp {
            use super::super::super::*;
            #[test]
            fn default() {
                run(
                    concat!("render::components::", stringify!($comp), "::default"),
                    $c,
                    St::Default,
                );
            }
            #[test]
            fn focused() {
                run(
                    concat!("render::components::", stringify!($comp), "::focused"),
                    $c,
                    St::Focused,
                );
            }
            #[test]
            fn hovered() {
                run(
                    concat!("render::components::", stringify!($comp), "::hovered"),
                    $c,
                    St::Hovered,
                );
            }
            #[test]
            fn pressed() {
                run(
                    concat!("render::components::", stringify!($comp), "::pressed"),
                    $c,
                    St::Pressed,
                );
            }
            #[test]
            fn disabled() {
                run(
                    concat!("render::components::", stringify!($comp), "::disabled"),
                    $c,
                    St::Disabled,
                );
            }
            #[test]
            fn selected() {
                run(
                    concat!("render::components::", stringify!($comp), "::selected"),
                    $c,
                    St::Selected,
                );
            }
            #[test]
            fn editing() {
                run(
                    concat!("render::components::", stringify!($comp), "::editing"),
                    $c,
                    St::Editing,
                );
            }
            #[test]
            fn empty() {
                run(
                    concat!("render::components::", stringify!($comp), "::empty"),
                    $c,
                    St::Empty,
                );
            }
        }
    };
}

/// A pair of readiness states that is **known** to render identically, and
/// must stay that way: `(component, a, b, colour)`.
///
/// **This list inverts.** A listed pair that turns out to be *distinct* is a
/// failure, not a pass. An exemption is a recorded fact about the design; a
/// stale one is how a gate quietly stops looking at the thing it was written
/// for, which is the defect class this whole matrix has spent the refactor
/// cataloguing.
///
/// The four entries record one fact: `BUSY` and `LOADING` both mean *an
/// operation is in flight*, every one of these components paints the same
/// `design.motion.spinner_frames[0]` affordance for the union
/// `BUSY | LOADING`, and no `junie`/`paper` rule separates them by colour. They
/// diverge only under `mono`, where §11.4 gives `PRESSED` and `EDITING` their
/// own symbols — which is why the colour is part of the key here and why the
/// mono half of each pair is *not* exempt.
const READINESS_COLLISIONS: [(&str, Status, Status, &str); 4] = [
    ("status_bar", Status::Busy, Status::Loading, "truecolor"),
    ("hint_bar", Status::Busy, Status::Loading, "truecolor"),
    ("progress_bar", Status::Busy, Status::Loading, "truecolor"),
    ("meter", Status::Busy, Status::Loading, "truecolor"),
];

/// The four readiness values and the matrix state that carries each.
const READINESS: [(Status, St); 4] = [
    (Status::Ready, St::Default),
    (Status::Busy, St::Pressed),
    (Status::Loading, St::Editing),
    (Status::Error, St::Disabled),
];

const THEME_NAMES: [&str; 2] = ["junie", "paper"];
const COLOR_NAMES: [&str; 2] = ["truecolor", "mono"];
const SIZES: [(u16, u16); 2] = [(120, 40), (40, 10)];

/// `tests/baselines/components.txt` as `key -> hash`, where the key is the
/// line minus its last whitespace field — `Baseline`'s own rule, and the one
/// `xtask bless-guard` uses, so all three agree about what a key is.
fn baseline_entries() -> std::collections::BTreeMap<String, String> {
    let text = std::fs::read_to_string(BASELINE.path())
        .unwrap_or_else(|e| panic!("read {}: {e}", BASELINE.path()));
    let mut out = std::collections::BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, hash)) = line.rsplit_once(' ') {
            out.insert(key.to_owned(), hash.to_owned());
        }
    }
    out
}

/// Every readiness cell of the matrix as `key -> hash`, rendered by the code
/// in this tree. `BLESS=1` writes exactly these values, so this is what the
/// baseline will hold after the next bless.
fn live_entries() -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for (name, comp) in Comp::ALL {
        if comp.status_prop(St::Default).is_none() {
            continue;
        }
        for (_, st) in READINESS {
            for (theme_name, theme) in [("junie", Theme::junie()), ("paper", Theme::paper())] {
                for (color_name, color) in [
                    ("truecolor", ColorLevel::TrueColor),
                    ("mono", ColorLevel::Mono),
                ] {
                    for (w, h) in SIZES {
                        let mut scene = Scene::new("readiness", theme.clone(), color, w, h);
                        scene.draw(|ui, area| draw(comp, st, ui, area));
                        out.insert(
                            format!(
                                "render::components::{name}::{} {w} {h} {theme_name} {color_name}",
                                st.name()
                            ),
                            format!("{:016x}", scene.digest()),
                        );
                    }
                }
            }
        }
    }
    out
}

/// Every readiness pair of `entries` that violates §49.5, as a report line.
///
/// Separated from the test so the same rule can be run against the recorded
/// baseline, against freshly rendered digests, and against deliberately broken
/// inputs. A missing key is a failure, never a skip.
fn readiness_failures(entries: &std::collections::BTreeMap<String, String>) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let mut used = [false; READINESS_COLLISIONS.len()];
    for (name, comp) in Comp::ALL {
        if comp.status_prop(St::Default).is_none() {
            continue;
        }
        for theme in THEME_NAMES {
            for color in COLOR_NAMES {
                for (w, h) in SIZES {
                    for (i, (sa, st_a)) in READINESS.iter().enumerate() {
                        for (sb, st_b) in READINESS.iter().skip(i + 1) {
                            let cell = format!("{w} {h} {theme} {color}");
                            let ka = format!("render::components::{name}::{} {cell}", st_a.name());
                            let kb = format!("render::components::{name}::{} {cell}", st_b.name());
                            let (Some(ha), Some(hb)) = (entries.get(&ka), entries.get(&kb)) else {
                                failures.push(format!(
                                    "  {name} {cell}: no baseline line for `{ka}` or `{kb}`"
                                ));
                                continue;
                            };
                            let exempt = READINESS_COLLISIONS.iter().position(|(c, x, y, l)| {
                                *c == name
                                    && *l == color
                                    && ((x == sa && y == sb) || (x == sb && y == sa))
                            });
                            match exempt {
                                Some(idx) => {
                                    used[idx] = true;
                                    if ha != hb {
                                        failures.push(format!(
                                            "  {name} {cell}: `{sa:?}` and `{sb:?}` are listed in \
                                             READINESS_COLLISIONS as indistinguishable but their \
                                             digests differ ({ha} vs {hb}); the exemption is \
                                             stale — remove it, the list inverts"
                                        ));
                                    }
                                }
                                None => {
                                    if ha == hb {
                                        failures.push(format!(
                                            "  {name} {cell}: `{sa:?}` and `{sb:?}` share the \
                                             digest {ha}; the fixture supplies a different \
                                             `Status` for each, so this cell pins two states of \
                                             one component as one picture"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for (i, (name, a, b, color)) in READINESS_COLLISIONS.iter().enumerate() {
        if !used[i] {
            failures.push(format!(
                "  READINESS_COLLISIONS lists ({name}, {a:?}, {b:?}, {color}), which names no cell \
                 of the matrix; a listed pair that is never reached exempts nothing and hides the \
                 next one that is"
            ));
        }
    }
    failures
}

mod theme {
    use super::*;

    /// §20.10 item 20 / §49.5. For every component the matrix drives with a
    /// `Status` prop, the `Ready`, `Busy`, `Loading` and `Error` digests are
    /// **pairwise distinct** at every size, theme and colour level.
    ///
    /// **Why this is asserted about the recorded values and not about a fresh
    /// render, and why that generalises.** A first-generation baseline line is
    /// unprotected by any diff-based gate: `xtask bless-guard` compares a tree
    /// with its base, and the *first* recording of a key has no before-image,
    /// so there is nothing for a diff to refuse. That is exactly how §20.10
    /// item 19's 896 lines came to pin `progress_bar::disabled` as a bar that
    /// is in error and paints no error glyph — at `truecolor` all four of its
    /// `::disabled` digests were byte-identical to its own `::default` ones,
    /// although the fixture supplies `Status::Error` to one and `Status::Ready`
    /// to the other. A component declaring `Caps::REPORTS_STATUS` reporting
    /// nothing. **So the properties that pin a first generation have to be
    /// asserted about the values themselves**, read out of the baseline file,
    /// rather than about the movement of those values.
    ///
    /// Item 19's own review could not have caught it and did not claim to: its
    /// six rejection conditions include textual identity only under `mono`,
    /// and these cells are identical at `truecolor`.
    ///
    /// The set of components checked comes from [`Comp::status_prop`], not
    /// from a list written here, and every failing cell is reported rather
    /// than only the first — §49.1 records what reading a single panic as "the
    /// moved set" cost.
    #[test]
    fn readiness_states_are_digest_distinct() {
        let entries = baseline_entries();

        // A component registered by `matrix!` but missing from `Comp::ALL`
        // would be silently unchecked. It cannot be: the baseline holds
        // 8 states × 2 themes × 2 colours × 2 sizes = 64 lines per component.
        let recorded = entries
            .keys()
            .filter(|k| k.starts_with("render::components::"))
            .count();
        assert_eq!(
            recorded,
            Comp::ALL.len() * 64,
            "the baseline holds {recorded} `render::components::` lines but `Comp::ALL` accounts \
             for {} components × 64; a component registered in `matrix!` is missing from \
             `Comp::ALL` and would go unchecked here",
            Comp::ALL.len()
        );

        // the state each readiness value is carried by must be the state the
        // fixture actually maps onto it
        for (status, st) in READINESS {
            assert_eq!(
                Comp::ProgressBar.status_prop(st),
                Some(status),
                "`READINESS` says `{st:?}` carries `{status:?}`, `status_for` disagrees"
            );
        }

        let failures = readiness_failures(&entries);
        assert!(
            failures.is_empty(),
            "{} readiness cell(s) are not digest-distinct (§20.10 item 20, §49.5):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// The green half of the demonstration, run without a bless.
    ///
    /// `BLESS=1` writes exactly `Scene::digest()` for each key, so rendering
    /// every status-driven cell here produces the values the re-bless will
    /// record. The property must hold on those. That splits "the code is
    /// right" from "the file is right": this test is green today and
    /// `readiness_states_are_digest_distinct` is red, and the difference
    /// between them is the eight stale `truecolor` lines §49.1 identified.
    #[test]
    fn readiness_distinctness_holds_on_the_digests_the_code_produces_now() {
        let failures = readiness_failures(&live_entries());
        assert!(
            failures.is_empty(),
            "{} readiness cell(s) rendered by the current code are not digest-distinct:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// The check can fail in both of its directions, on inputs built here.
    ///
    /// COORDINATION.md: a check that has never been observed red is not
    /// evidence. The inverted exemption is the half that is easy to get wrong,
    /// because a stale entry silently stops the gate looking at a real pair.
    #[test]
    fn readiness_distinctness_fails_on_a_collision_and_on_a_stale_exemption() {
        let mut collided = live_entries();
        let ready = "render::components::meter::default 120 40 junie truecolor";
        let error = "render::components::meter::disabled 120 40 junie truecolor";
        let v = collided.get(ready).cloned().expect("a rendered Ready cell");
        collided.insert(error.to_owned(), v);
        let failures = readiness_failures(&collided);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("Ready"), "{failures:?}");
        assert!(failures[0].contains("Error"), "{failures:?}");

        let mut split = live_entries();
        let busy = "render::components::meter::pressed 120 40 junie truecolor";
        split.insert(busy.to_owned(), "0000000000000000".to_owned());
        let failures = readiness_failures(&split);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(
            failures[0].contains("the exemption is stale"),
            "{failures:?}"
        );
    }

    /// A missing key is a failure, never a skip: an exemption list that
    /// silently passes over absent cells is a gate that stops looking.
    #[test]
    fn readiness_distinctness_fails_on_a_missing_baseline_line() {
        let mut entries = live_entries();
        entries.remove("render::components::hint_bar::disabled 40 10 paper mono");
        let failures = readiness_failures(&entries);
        assert_eq!(
            failures.len(),
            3,
            "one per pair the missing key is in: {failures:?}"
        );
        assert!(failures[0].contains("no baseline line"), "{failures:?}");
    }
}

mod render {
    mod components {
        matrix!(button, Comp::Button);
        matrix!(text_input, Comp::TextInput);
        matrix!(field, Comp::Field);
        matrix!(list, Comp::List);
        matrix!(tabs, Comp::Tabs);
        matrix!(dialog, Comp::Dialog);
        matrix!(text_area, Comp::TextArea);
        matrix!(select, Comp::Select);
        matrix!(radio_group, Comp::RadioGroup);
        matrix!(checkbox, Comp::Checkbox);
        matrix!(toggle, Comp::Toggle);
        matrix!(chip_bar, Comp::ChipBar);
        matrix!(status_bar, Comp::StatusBar);
        matrix!(hint_bar, Comp::HintBar);
        matrix!(key_hint, Comp::KeyHint);
        matrix!(progress_bar, Comp::ProgressBar);
        matrix!(spinner, Comp::Spinner);
        matrix!(meter, Comp::Meter);
        matrix!(empty, Comp::Empty);
        matrix!(brand, Comp::Brand);
    }
}
