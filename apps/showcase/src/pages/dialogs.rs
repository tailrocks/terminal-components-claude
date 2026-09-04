//! Modal confirmation and prompt flows.

use junie_tui::{
    ActionKey, Button, Cx, Dialog, DialogAction, DialogState, Id, Rect, Response, Ui, Variant, id,
    layout,
};

use super::{Page, frame, lines};

const OPEN_CONFIRM: Id = id!("dialogs.confirm.open");
const OPEN_PROMPT: Id = id!("dialogs.prompt.open");
const CONFIRM: Id = id!("dialogs.confirm.layer");
const PROMPT: Id = id!("dialogs.prompt.layer");
const DIALOG_LABEL_PATCH: junie_tui::StylePatch = junie_tui::StylePatch::new()
    .set_fg(junie_tui::Role::Accent)
    .add(junie_tui::Modifier::BOLD);
const DIALOG_PARTS: &[(junie_tui::Part, junie_tui::StylePatch)] =
    &[(junie_tui::Part::TITLE, DIALOG_LABEL_PATCH)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpenDialog {
    None,
    Confirm,
    Prompt,
}

/// Dialog launchers and their durable prompt states.
#[derive(Debug)]
pub(crate) struct DialogsPage {
    open: OpenDialog,
    confirm_state: DialogState,
    prompt_state: DialogState,
    result: String,
}

impl DialogsPage {
    pub(crate) fn new() -> Self {
        Self {
            open: OpenDialog::None,
            confirm_state: DialogState::default(),
            prompt_state: DialogState::default(),
            result: String::from("none"),
        }
    }

    fn confirm() -> Dialog<'static> {
        Dialog::confirm(
            CONFIRM,
            "Run task now?",
            "The task will be queued for the workspace.",
        )
        .patch_part(DIALOG_PARTS)
    }

    fn prompt() -> Dialog<'static> {
        Dialog::prompt(PROMPT, "Rename task", "Task name").patch_part(DIALOG_PARTS)
    }

    fn close(&mut self, cx: &mut Cx<'_>, id: Id) {
        if cx.is_open(id) {
            cx.close_layer(id, None);
        }
        self.open = OpenDialog::None;
    }
}

impl Default for DialogsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for DialogsPage {
    fn title(&self) -> &'static str {
        "Dialogs"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let confirm_button = Button::new(OPEN_CONFIRM, "Run task now")
            .variant(Variant::PRIMARY)
            .update(cx);
        if confirm_button.activated() && !cx.is_open(CONFIRM) {
            self.open = OpenDialog::Confirm;
            cx.open_layer(CONFIRM, Self::confirm().layer(cx));
        }
        response |= confirm_button.erase();
        let prompt_button = Button::new(OPEN_PROMPT, "Rename task")
            .variant(Variant::SECONDARY)
            .update(cx);
        if prompt_button.activated() && !cx.is_open(PROMPT) {
            self.open = OpenDialog::Prompt;
            cx.open_layer(PROMPT, Self::prompt().layer(cx));
        }
        response |= prompt_button.erase();

        // Update layers unconditionally. A dismissed layer is removed by the
        // runtime before the app update, and Dialog drains that dismissal
        // action from its durable state on the following frame.
        let action = Self::confirm().update(cx, &mut self.confirm_state);
        if self.open == OpenDialog::Confirm
            && let Some(action) = action.action_ref()
        {
            match action {
                DialogAction::Action(key) if *key == ActionKey::CONFIRM => {
                    self.result = String::from("Task started");
                    self.close(cx, CONFIRM);
                }
                DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                    self.result = String::from("Cancelled");
                    self.close(cx, CONFIRM);
                }
            }
        }
        response |= action.erase();
        let action = Self::prompt().update(cx, &mut self.prompt_state);
        if self.open == OpenDialog::Prompt
            && let Some(action) = action.action_ref()
        {
            match action {
                DialogAction::Action(key) if *key == ActionKey::CONFIRM => {
                    let name = self.prompt_state.draft().trim();
                    if name.is_empty() {
                        self.result = String::from("Name cannot be empty");
                    } else {
                        self.result = format!("Task: {name}");
                        self.close(cx, PROMPT);
                    }
                }
                DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                    self.result = String::from("Rename cancelled");
                    self.close(cx, PROMPT);
                }
            }
        }
        response |= action.erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "layers · focus trap · Esc dismiss",
            |ui, body| {
                let (actions, status) = layout::split_v(body, 4);
                let action_rows = super::rows(actions, 2);
                Button::new(OPEN_CONFIRM, "Run task now")
                    .variant(Variant::PRIMARY)
                    .draw(ui, action_rows.first().copied().unwrap_or(actions));
                Button::new(OPEN_PROMPT, "Rename task")
                    .variant(Variant::SECONDARY)
                    .draw(ui, action_rows.get(1).copied().unwrap_or(actions));
                let _ = ui.paint_str(status, &self.result, ui.surface_style());
                lines(
                    ui,
                    Rect {
                        y: status.y.saturating_add(1),
                        height: status.height.saturating_sub(1),
                        ..status
                    },
                    &[
                        "A modal traps focus in its prompt/actions and restores the launcher.",
                        "Prompt submission rejects empty values before closing the layer.",
                    ],
                );
            },
        );
        ui.layer(CONFIRM, |ui, layer| {
            Self::confirm().draw(ui, layer, &self.confirm_state, |ui, body| {
                let _ = ui.paint_str(body, "Enter confirms · Esc cancels", ui.surface_style());
            });
        });
        ui.layer(PROMPT, |ui, layer| {
            Self::prompt().draw(ui, layer, &self.prompt_state, |ui, body| {
                let _ = ui.paint_str(body, "Type a name, then Enter", ui.surface_style());
            });
        });
    }
}
