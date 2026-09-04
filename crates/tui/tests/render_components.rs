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

fn status_for(st: St) -> Status {
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

fn draw_status_bar(st: St, ui: &mut Ui<'_>, area: Rect) {
    let left: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_LEFT };
    let center: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_CENTER };
    let right: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_RIGHT };
    StatusBar::new(STATUS_BAR)
        .left(left)
        .center(center)
        .right(right)
        .status(status_for(st))
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_hint_bar(st: St, ui: &mut Ui<'_>, area: Rect) {
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
        .status(status_for(st))
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_key_hint(st: St, ui: &mut Ui<'_>, area: Rect) {
    KeyHint::new(KEY_HINT, Chord::key(KeyCode::Enter), "Open")
        .state_override(st.flags())
        .draw(ui, area);
}

fn draw_progress_bar(st: St, ui: &mut Ui<'_>, area: Rect) {
    ProgressBar::new(PROGRESS_BAR)
        .label(if st.is_empty() { "" } else { "Uploading" })
        .ratio(if st.is_empty() { 0.0 } else { 0.65 })
        .status(status_for(st))
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

fn draw_meter(st: St, ui: &mut Ui<'_>, area: Rect) {
    Meter::new(METER)
        .ratio(if st.is_empty() { 0.0 } else { 0.65 })
        .value(if st.is_empty() { "" } else { "65%" })
        .status(status_for(st))
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
        Comp::StatusBar => draw_status_bar(st, ui, area),
        Comp::HintBar => draw_hint_bar(st, ui, area),
        Comp::KeyHint => draw_key_hint(st, ui, area),
        Comp::ProgressBar => draw_progress_bar(st, ui, area),
        Comp::Spinner => draw_spinner(st, ui, area),
        Comp::Meter => draw_meter(st, ui, area),
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
