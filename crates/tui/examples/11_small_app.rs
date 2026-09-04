//! `COMPONENT_ARCHITECTURE.md` §17 example 11, verbatim (crate name is temporary: `tui_next` → `junie_tui` at Slice 5).
#![expect(clippy::indexing_slicing, reason = "verbatim from §17 example 11")]

use tui_next::{
    Action, ActionKey, App, Button, Cx, Dialog, DialogAction, DialogState, Field, FrameRead, Id,
    Insets, ItemKey, List, ListAction, ListState, Response, RowUi, TextInput, TextInputState,
    Theme, Track, Ui, Variant, id, layout, run,
};

const NAME: Id = id!("name");
const ADD: Id = id!("add");
const PEOPLE: Id = id!("people");
const CONFIRM: Id = id!("confirm");
const K_YES: ActionKey = ActionKey::CONFIRM;
const K_NO: ActionKey = ActionKey::CANCEL;

#[derive(Default)]
struct Roster {
    name: String,
    name_st: TextInputState,
    people: Vec<String>,
    list: ListState,
    dlg: DialogState,
    pending_remove: Option<ItemKey>,
    quit: bool,
}

// One constructor per configured control, called from both phases (§13 "props are built once").
// Each takes the fields it needs as parameters, never `&self`, so `update` keeps `&mut` access.
fn add_button(name_empty: bool) -> Button<'static> {
    Button::new(ADD, "Add")
        .variant(Variant::PRIMARY)
        .disabled(name_empty)
}
fn name_input() -> TextInput<'static> {
    TextInput::new(NAME)
}
fn people_list()
-> List<'static, String, impl Fn(&String) -> ItemKey, impl Fn(&String, &mut RowUi<'_>)> {
    List::new(PEOPLE)
        .key(|s: &String| ItemKey::text(s))
        .row(|s: &String, u: &mut RowUi<'_>| u.label(s))
}
// The action row belongs to the props: `measured_height` sizes the layer from
// the same `Dialog` the frame draws, so the two cannot disagree (§26 N1).
const REMOVE_ACTIONS: [Action<'static>; 2] =
    [Action::new(K_NO, "Cancel"), Action::danger(K_YES, "Remove")];
fn remove_dialog() -> Dialog<'static> {
    Dialog::destructive(
        CONFIRM,
        "Remove person",
        "Remove this person from the roster?",
    )
    .actions(&REMOVE_ACTIONS)
    .cancel(K_NO)
}

impl App for Roster {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();

        r |= name_input()
            .update(cx, &mut self.name_st, &mut self.name)
            .erase();

        r |= add_button(self.name.trim().is_empty())
            .update(cx)
            .on_activated(|| {
                self.people.push(std::mem::take(&mut self.name));
            });

        r |= people_list()
            .update(cx, &mut self.list, &self.people) // items per phase (§21 item 1)
            .on_action(|a| {
                if let ListAction::Activated(k) = a {
                    self.pending_remove = Some(k);
                    // the component sizes its own layer (§26 N1)
                    cx.open_layer(CONFIRM, remove_dialog().layer(cx));
                }
            });

        // §13: a component that owns a layer runs its `update`
        // **unconditionally**, every frame, open or not. `cx.is_open` guards
        // the work the *caller* does besides the component, never the
        // component's own `update`: the dismissal is delivered as intents
        // addressed to the layer's owner in the pass **after** the layer
        // closed, so a gated call would drain nothing and drop it.
        r |= remove_dialog().update(cx, &mut self.dlg).on_action(|a| {
            if let DialogAction::Action(K_YES) = a
                && let Some(k) = self.pending_remove.take()
            {
                self.people.retain(|s| ItemKey::text(s) != k);
            }
            if cx.is_open(CONFIRM) {
                cx.close_layer(CONFIRM, None);
            }
        });
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let body = layout::inset(
            ui.full(),
            Insets {
                l: 2,
                t: 1,
                r: 2,
                b: 1,
            },
        );
        let rows = layout::rows(body, &[Track::Fixed(3), Track::Fixed(1), Track::Flex(1)]);
        let top = layout::columns(
            rows[0],
            &[Track::Flex(1), Track::Fixed(10)],
            ui.design().space.gap,
        );

        Field::new("Name", name_input().value(&self.name)).draw(ui, top[0], &self.name_st);
        add_button(self.name.trim().is_empty()).draw(ui, top[1]);
        people_list().draw(ui, rows[2], &self.list, &self.people);

        ui.layer(CONFIRM, |ui, a| {
            remove_dialog().draw(ui, a, &self.dlg, |_, _| {});
        });
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn main() -> std::io::Result<()> {
    run(Roster::default(), Theme::junie())
}
