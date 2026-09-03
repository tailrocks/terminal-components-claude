//! `COMPONENT_ARCHITECTURE.md` §17 example 9, verbatim (crate name is temporary: `tui_next` → `junie_tui` at Slice 5).
#![expect(
    dead_code,
    clippy::unnested_or_patterns,
    clippy::indexing_slicing,
    clippy::unused_self,
    clippy::semicolon_if_nothing_returned,
    reason = "verbatim from §17 example 9"
)]

use tui_next::{
    Action, ActionKey, Cx, Dialog, DialogAction, DialogState, DismissReason, Id, LayerSpec, Part,
    Props, Response, TextInput, TextInputState, Track, Ui, id, layout,
};

const CONFIRM: Id = id!("confirm.delete");
const TOKEN: Id = CONFIRM.part(Part::FIELD); // a child COMPONENT id inside the dialog (§21 item 16)
const K_CANCEL: ActionKey = ActionKey::CANCEL;
const K_DELETE: ActionKey = ActionKey::custom("delete");

struct Screen {
    dlg: DialogState,
    token: String,
    token_st: TextInputState,
    target: String,
    deleted: bool,
}

impl Screen {
    fn open(&mut self, cx: &mut Cx<'_>) {
        cx.open_layer(CONFIRM, LayerSpec::modal(CONFIRM));
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = TextInput::new(TOKEN)
            .update(cx, &mut self.token_st, &mut self.token)
            .erase();

        let armed = self.token.trim() == self.target; // arming is an `update` predicate
        let actions = [
            Action::new(K_CANCEL, "Cancel"),
            Action::danger(K_DELETE, "Delete").enabled(armed),
        ];
        let d = Dialog::new(CONFIRM)
            .title("Delete table")
            .description("This cannot be undone.")
            .actions(&actions)
            .cancel(K_CANCEL)
            .update(cx, &mut self.dlg);

        match d.action_ref() {
            Some(DialogAction::Action(k)) if *k == K_DELETE => {
                self.deleted = true;
                cx.close_layer(CONFIRM, Some(K_DELETE));
            }
            Some(DialogAction::Action(_)) | Some(DialogAction::Dismissed(DismissReason::Esc)) => {
                cx.close_layer(CONFIRM, Some(K_CANCEL))
            }
            _ => {}
        }
        r |= d.erase();
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(CONFIRM, |ui, area| {
            Dialog::new(CONFIRM).title("Delete table").width(60).draw(
                ui,
                area,
                &self.dlg,
                |ui, body| {
                    // ARBITRARY body content
                    let rows = layout::rows(body, &[Track::Auto, Track::Fixed(1), Track::Flex(1)]);
                    Props::new(&[("Table", self.target.as_str()), ("Rows", "12,481")])
                        .draw(ui, rows[0]);
                    ui.rule(rows[1]);
                    TextInput::new(TOKEN)
                        .value(&self.token)
                        .placeholder("Type the table name to confirm")
                        .draw(ui, rows[2], &self.token_st);
                },
            );
        });
    }
}

fn main() {}
// `DialogBody` does not exist. The body is a closure that borrows application data.
// Focus trapping, backdrop, Esc, click-outside, focus restore and the hint layer come
// from the layer, not from the dialog. Esc reaches the editing `TextInput` first and the
// layer only afterwards (§21 item 3).
