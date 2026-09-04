//! The component digest matrix — `render::components::<component>::<state>`
//! (`COMPONENT_ARCHITECTURE.md` §16.3, Slice 2 acceptance condition 5).
//!
//! Six components × eight states × `{junie, paper}` × `{truecolor, mono}` ×
//! `{120×40, 40×10}` = 384 checked-in digest lines in
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
    Action, ActionKey, Button, ColorLevel, Dialog, DialogState, Field, Id, ItemKey, List,
    ListState, Rect, RowUi, StateFlags, Tabs, TabsState, TextInput, TextInputState, Theme, Ui,
    Variant,
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

/// The six components the matrix covers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Comp {
    Button,
    TextInput,
    Field,
    List,
    Tabs,
    Dialog,
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

/// Draw `comp` in state `st` into `area`.
fn draw(comp: Comp, st: St, ui: &mut Ui<'_>, area: Rect) {
    let flags = st.flags();
    match comp {
        Comp::Button => {
            let label = if st.is_empty() { "" } else { "Run task" };
            Button::new(BTN, label)
                .variant(Variant::PRIMARY)
                .state_override(flags)
                .draw(ui, area);
        }
        Comp::TextInput => {
            text_input(INPUT, st).draw(ui, area, &TextInputState::default());
        }
        Comp::Field => {
            let label = if st.is_empty() { "" } else { "Name" };
            let mut f = Field::new(label, text_input(FIELD, st))
                .required(true)
                .state_override(flags);
            if !st.is_empty() {
                f = f.help("The person's display name.");
            }
            f.draw(ui, area, &TextInputState::default());
        }
        Comp::List => {
            let key: fn(&(&str, &str)) -> ItemKey = row_key;
            let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
            let items: &[(&str, &str)] = if st.is_empty() { &[] } else { &ROWS };
            List::new(LIST)
                .key(key)
                .row(row)
                .state_override(flags)
                .draw(ui, area, &ListState::default(), items);
        }
        Comp::Tabs => {
            let key: fn(&&'static str) -> ItemKey = tab_key;
            let row: fn(&&'static str, &mut RowUi<'_>) = tab_paint;
            let items: &[&'static str] = if st.is_empty() { &[] } else { &TAB_LABELS };
            Tabs::new(TABS)
                .key(key)
                .row(row)
                .closable(true)
                .allow_new(true)
                .state_override(flags)
                .draw(ui, area, &TabsState::default(), items);
        }
        Comp::Dialog => {
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
            d.state_override(flags)
                .draw(ui, area, &DialogState::default(), |_, _| {});
        }
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
    }
}
