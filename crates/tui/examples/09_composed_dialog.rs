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
    Action, ActionKey, Constraints, Cx, Dialog, DialogAction, DialogState, DismissReason, Id, Part,
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

// The single props constructor §13 requires (`architecture::props_are_built_once`). It is also what
// sizes the layer: `body_rows` states what the body slot needs, and `Dialog::layer` turns
// `(props, DesignTokens)` into `LayerSpec::modal(CONFIRM).size(LayerSize::Fixed(w, h))` (§26 N1).
fn confirm_dialog() -> Dialog<'static> {
    Dialog::new(CONFIRM)
        .title("Delete table")
        .description("This cannot be undone.")
        .width(60)
        .body_rows(4) // props (2) + rule (1) + token field (1)
}

impl Screen {
    fn open(&mut self, cx: &mut Cx<'_>) {
        cx.open_layer(CONFIRM, confirm_dialog().layer(cx));
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
        // `Dialog::update` re-asserts the layer size first (invariant D1), so a longer description or a
        // taller body corrects the layer on the next draw without the opener predicting anything.
        let d = confirm_dialog()
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
            confirm_dialog().draw(ui, area, &self.dlg, |ui, body| {
                // ARBITRARY body content
                // `Track::Auto` without a measurement is ONE cell when explicit `Flex` tracks exist
                // (§10, §25 adjudication 7), which would clip this two-row `Props`. Supply the
                // natural size: that is what `rows_measured` is for.
                // FINDING vs §17 example 9: `Props::new(&[…])` borrows a temporary
                // array, so binding `props` for a later `measure` needs the array
                // named first (E0716). One added line; nothing else changes.
                let fields = [("Table", self.target.as_str()), ("Rows", "12,481")];
                let props = Props::new(&fields);
                let natural = [props
                    .measure(ui, Constraints::loose(body.width, body.height))
                    .preferred
                    .1];
                let rows = layout::rows_measured(
                    body,
                    &[Track::Auto, Track::Fixed(1), Track::Flex(1)],
                    &natural,
                );
                props.draw(ui, rows[0]);
                ui.rule(rows[1]);
                TextInput::new(TOKEN)
                    .value(&self.token)
                    .placeholder("Type the table name to confirm")
                    .draw(ui, rows[2], &self.token_st);
            });
        });
    }
}

fn main() {}
// `DialogBody` does not exist. The body is a closure that borrows application data.
// Focus trapping, backdrop, Esc, click-outside, focus restore and the hint layer come
// from the layer, not from the dialog. Esc reaches the editing `TextInput` first and the
// layer only afterwards (§21 item 3).
// The dialog computes a SIZE, never a rect: placement, flip and clamp stay in the one
// resolver (§9.1, §26 N1). `Rect::centered*` appears nowhere in a component.
