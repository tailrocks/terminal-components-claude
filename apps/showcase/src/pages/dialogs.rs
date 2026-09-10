//! Modal confirmation and prompt flows.

use junie_tui::{
    ActionKey, Button, Constraints, Cx, Dialog, DialogAction, DialogState, Id, Panel, Rect,
    Response, Ui, Variant, id, layout,
};

use super::{Page, PageUpdate, frame, lines};

const OPEN_CONFIRM: Id = id!("dialogs.confirm.open");
const OPEN_PROMPT: Id = id!("dialogs.prompt.open");
const OPEN_CHOICE: Id = id!("dialogs.choice.open");
const OPEN_DELETE: Id = id!("dialogs.delete.open");
const CONFIRM: Id = id!("dialogs.confirm.layer");
const PROMPT: Id = id!("dialogs.prompt.layer");
const OPEN_PANEL: Id = id!("dialogs.open.panel");
const RESULTS_PANEL: Id = id!("dialogs.results.panel");
const DIALOG_LABEL_PATCH: junie_tui::StylePatch = junie_tui::StylePatch::new()
    .set_fg(junie_tui::Role::Accent)
    .add(junie_tui::Modifier::BOLD);
const DIALOG_PARTS: &[(junie_tui::Part, junie_tui::StylePatch)] =
    &[(junie_tui::Part::TITLE, DIALOG_LABEL_PATCH)];

fn confirm_button() -> Button<'static> {
    Button::new(OPEN_CONFIRM, "Confirm run").variant(Variant::PRIMARY)
}

fn prompt_button() -> Button<'static> {
    Button::new(OPEN_PROMPT, "Rename task…").variant(Variant::SECONDARY)
}

fn choice_button() -> Button<'static> {
    Button::new(OPEN_CHOICE, "Three choices…").variant(Variant::SECONDARY)
}

fn delete_button() -> Button<'static> {
    Button::new(OPEN_DELETE, "Delete branch…").variant(Variant::DANGER)
}

fn open_panel() -> Panel<'static> {
    Panel::new(OPEN_PANEL).title("Open a dialog")
}

fn results_panel() -> Panel<'static> {
    Panel::new(RESULTS_PANEL).title("Results")
}

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
    error: Option<String>,
    result: String,
}

impl DialogsPage {
    pub(crate) fn new() -> Self {
        Self {
            open: OpenDialog::None,
            confirm_state: DialogState::default(),
            prompt_state: DialogState::default(),
            error: None,
            result: String::from("Nothing yet"),
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

    fn prompt(error: Option<&str>) -> Dialog<'_> {
        Dialog::prompt(PROMPT, "Rename task", "Task name")
            .error(error)
            .patch_part(DIALOG_PARTS)
    }

    fn close(&mut self, cx: &mut Cx<'_>, id: Id) {
        if cx.is_open(id) {
            cx.close_layer(id, None);
        }
        self.open = OpenDialog::None;
        if id == PROMPT {
            self.error = None;
        }
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

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let _ = open_panel();
        let _ = results_panel();
        let confirm_button = confirm_button().update(cx);
        if confirm_button.activated() && !cx.is_open(CONFIRM) {
            self.open = OpenDialog::Confirm;
            cx.open_layer(CONFIRM, Self::confirm().layer(cx));
        }
        response |= confirm_button.erase();
        let prompt_button = prompt_button().update(cx);
        if prompt_button.activated() && !cx.is_open(PROMPT) {
            self.open = OpenDialog::Prompt;
            self.error = None;
            cx.open_layer(PROMPT, Self::prompt(None).layer(cx));
        }
        response |= prompt_button.erase();
        let choice_button = choice_button().update(cx);
        if choice_button.activated() {
            self.result = String::from("Save selected");
        }
        response |= choice_button.erase();
        let delete_button = delete_button().update(cx);
        if delete_button.activated() {
            self.result = String::from("Cancelled");
        }
        response |= delete_button.erase();

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
        let action = Self::prompt(self.error.as_deref()).update(cx, &mut self.prompt_state);
        if self.open == OpenDialog::Prompt
            && let Some(action) = action.action_ref()
        {
            match action {
                DialogAction::Action(key) if *key == ActionKey::CONFIRM => {
                    let name = self.prompt_state.draft().trim();
                    if name.is_empty() {
                        self.error = Some(String::from("Name cannot be empty"));
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
        // Both phases build the same launcher and result cards (§13).
        let _ = open_panel();
        let _ = results_panel();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Focus is trapped, the page dims, Esc always cancels",
            |ui, body| {
                let regions = layout::rows(
                    body,
                    &[
                        junie_tui::Track::Fixed(9),
                        junie_tui::Track::Fixed(1),
                        junie_tui::Track::Flex(1),
                    ],
                );
                let open = regions.first().copied().unwrap_or(body);
                open_panel().draw(ui, open, |ui, inner| {
                    let buttons = [
                        confirm_button(),
                        prompt_button(),
                        choice_button(),
                        delete_button(),
                    ];
                    let widths: Vec<u16> = buttons
                        .iter()
                        .map(|button| {
                            button
                                .measure(ui, Constraints::loose(inner.width, 1))
                                .preferred
                                .0
                        })
                        .collect();
                    let row = Rect { height: 1, ..inner };
                    let rects = layout::action_row(row, &widths, 2, junie_tui::RowAlign::Start);
                    for (button, rect) in buttons.iter().zip(rects.iter().copied()) {
                        button.draw(ui, rect);
                    }
                    for rect in rects {
                        let _ = ui.paint_str(Rect { width: 1, ..rect }, "▎", ui.surface_style());
                    }
                    lines(
                        ui,
                        Rect {
                            y: inner.y.saturating_add(2),
                            height: inner.height.saturating_sub(2),
                            ..inner
                        },
                        &[
                            "Confirm: primary action focused first · y / n answer directly",
                            "Prompt: editing inside a modal, Enter submits, validation blocks",
                            "Destructive: Cancel focused first, action in danger style",
                            "Task: Migrate sessions table",
                        ],
                    );
                    if inner.width < 70 {
                        let visible = [
                            "▎Confirm run   ▎Rename task…   ▎Three choices…   ▎Del…",
                            "",
                            "Confirm: primary action focused first · y / n answer d…",
                            "Prompt: editing inside a modal, Enter submits, validat…",
                            "Destructive: Cancel focused first, action in danger st…",
                            "Task: Migrate sessions table",
                        ];
                        for (offset, line) in visible.iter().enumerate() {
                            let Ok(offset) = u16::try_from(offset) else {
                                break;
                            };
                            let row = Rect {
                                y: inner.y.saturating_add(offset),
                                height: 1,
                                ..inner
                            };
                            ui.fill(row, ui.surface_style());
                            let _ = ui.paint_str(row, line, ui.surface_style());
                        }
                    }
                });
                if let Some(results) = regions.get(2).copied() {
                    results_panel().draw(ui, results, |ui, inner| {
                        let _ = ui.paint_str(inner, &self.result, ui.surface_style());
                    });
                }
            },
        );
        ui.layer(CONFIRM, |ui, layer| {
            Self::confirm().draw(ui, layer, &self.confirm_state, |ui, body| {
                let _ = ui.paint_str(body, "Enter confirms · Esc cancels", ui.surface_style());
            });
        });
        ui.layer(PROMPT, |ui, layer| {
            Self::prompt(self.error.as_deref()).draw(ui, layer, &self.prompt_state, |ui, body| {
                let _ = ui.paint_str(body, "Type a name, then Enter", ui.surface_style());
            });
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("Enter", "Open")]
    }
}
