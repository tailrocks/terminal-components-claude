//! `COMPONENT_ARCHITECTURE.md` §17 example 10 with `Picker` replaced by a `List` in a popover (crate name is temporary: `junie_tui` → `junie_tui` at Slice 5).
#![expect(
    dead_code,
    missing_docs,
    missing_debug_implementations,
    clippy::unused_self,
    clippy::semicolon_if_nothing_returned,
    reason = "verbatim from §17 example 10"
)]

use junie_tui::{
    Action, ActionKey, Anchor, Button, CrossAlign, Cx, Dialog, DialogState, Dismiss, FrameRead, Id,
    ItemKey, LayerEvent, LayerSpec, List, ListAction, ListState, Part, Response, RowUi, Side, Ui,
    id,
};

pub struct Person {
    pub id: u64,
    pub name: String,
    pub team: String,
}

const DLG: Id = id!("dlg");
const OWNER_BTN: Id = DLG.part(Part::custom("owner")); // a child COMPONENT id (§21 item 16, M5), not a PartRef
const OWNER_PICK: Id = id!("dlg.owner_picker");
const K_DONE: ActionKey = ActionKey::CONFIRM;

struct Screen {
    dlg: DialogState,
    pick: ListState,
    people: Vec<Person>,
    owner: Option<u64>,
}

fn dialog() -> Dialog<'static> {
    Dialog::new(DLG).title("Edit task").body_rows(1)
}

fn owner_picker()
-> List<'static, Person, impl Fn(&Person) -> ItemKey, impl Fn(&Person, &mut RowUi<'_>)> {
    List::new(OWNER_PICK)
        .key(|p: &Person| ItemKey::num(p.id))
        .row(|p: &Person, u: &mut RowUi<'_>| {
            u.label(&p.name);
            u.meta(&p.team);
        })
}

impl Screen {
    fn open(&mut self, cx: &mut Cx<'_>) {
        cx.open_layer(DLG, dialog().layer(cx));
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();

        let actions = [Action::new(K_DONE, "Done")];
        r |= dialog()
            .actions(&actions)
            .cancel(K_DONE)
            .update(cx, &mut self.dlg)
            .on_action(|_| cx.close_layer(DLG, Some(K_DONE)));

        // The picker opens ON TOP of the dialog as a popover anchored below the button: its
        // own focus scope and a pointer barrier, no full-screen dim (§21 item 8). The dialog
        // beneath is pointer- and key-inert until the picker closes.
        let anchor = cx.area(OWNER_BTN).unwrap_or_default();
        r |= Button::new(OWNER_BTN, "Choose owner…")
            .update(cx)
            .on_activated(|| {
                cx.open_layer(
                    OWNER_PICK,
                    LayerSpec::popover(
                        OWNER_PICK,
                        Anchor::Rect {
                            rect: anchor,
                            side: Side::Below,
                            align: CrossAlign::Start,
                        },
                    )
                    .dismiss(Dismiss::ESC_AND_OUTSIDE)
                    // the list's OWN arithmetic over the items it receives per
                    // phase (§26 N1); the opener re-asserts it below with
                    // `cx.resize_layer` while the popover is open
                    .size(owner_picker().measured_size(cx, &self.people)),
                )
            });

        if cx.is_open(OWNER_PICK) {
            let size = owner_picker().measured_size(cx, &self.people);
            cx.resize_layer(OWNER_PICK, size);
            r |= owner_picker()
                .update(cx, &mut self.pick, &self.people)
                .on_action(|a| {
                    if let ListAction::Chose(k) = a {
                        self.owner = self
                            .people
                            .iter()
                            .find(|p| ItemKey::num(p.id) == k)
                            .map(|p| p.id);
                        cx.close_layer(OWNER_PICK, Some(ActionKey::CONFIRM));
                    }
                });
        }

        if let Some(LayerEvent::Dismissed(_)) = cx.layer_event(OWNER_PICK) { /* nothing to undo */ }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(DLG, |ui, a| {
            dialog().draw(ui, a, &self.dlg, |ui, body| {
                Button::new(OWNER_BTN, "Choose owner…").draw(ui, body);
            });
        });
        ui.layer(OWNER_PICK, |ui, a| {
            owner_picker().draw(ui, a, &self.pick, &self.people);
        });
    }
}

fn main() {}
// Esc closes only the picker; the dialog stays open and regains focus at the button.
// No barrier is pushed by hand, no hit region is re-registered, and the picker draws no
// hint row of its own — the top layer contributes to the shared HintBar (§13.1).
// z-order is the `LayerId` assigned by `open_layer`, not the order of the two `ui.layer` calls (§21 item 14).
