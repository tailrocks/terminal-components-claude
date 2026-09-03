//! `COMPONENT_ARCHITECTURE.md` §17 example 11, verbatim (crate name is temporary: `tui_next` → `junie_tui` at Slice 5).
#![expect(
    clippy::indexing_slicing,
    clippy::collapsible_if,
    reason = "verbatim from §17 example 11"
)]

use tui_next::{
    Action, ActionKey, App, Button, Cx, Dialog, DialogAction, DialogState, Field, FrameRead, Id,
    Insets, ItemKey, LayerSpec, List, ListAction, ListState, Response, RowUi, TextInput,
    TextInputState, Theme, Track, Ui, Variant, id, layout, run,
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
fn people_list()
-> List<'static, String, impl Fn(&String) -> ItemKey, impl Fn(&String, &mut RowUi<'_>)> {
    List::new(PEOPLE)
        .key(|s: &String| ItemKey::text(s))
        .row(|s: &String, u: &mut RowUi<'_>| u.label(s))
}
fn remove_dialog() -> Dialog<'static> {
    Dialog::destructive(
        CONFIRM,
        "Remove person",
        "Remove this person from the roster?",
    )
}

impl App for Roster {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();

        r |= TextInput::new(NAME)
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
                    cx.open_layer(CONFIRM, LayerSpec::modal(CONFIRM));
                }
            });

        if cx.is_open(CONFIRM) {
            let actions = [Action::new(K_NO, "Cancel"), Action::danger(K_YES, "Remove")];
            r |= remove_dialog()
                .actions(&actions)
                .cancel(K_NO)
                .update(cx, &mut self.dlg)
                .on_action(|a| {
                    if let DialogAction::Action(K_YES) = a {
                        if let Some(k) = self.pending_remove.take() {
                            self.people.retain(|s| ItemKey::text(s) != k);
                        }
                    }
                    cx.close_layer(CONFIRM, None);
                });
        }
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

        Field::new("Name", TextInput::new(NAME).value(&self.name)).draw(ui, top[0], &self.name_st);
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
