//! Single-line controlled editing with commit, cancel and validation feedback.

use junie_tui::{
    BlurPolicy, Cx, Id, Panel, PanelKind, Rect, Response, TextAction, TextInput, TextInputState,
    Ui, id,
};

use super::{Page, frame, lines, rows};

const NAME: Id = id!("inputs.name");
const BRANCH: Id = id!("inputs.branch");
const CARD: Id = id!("inputs.card");

fn name_input() -> TextInput<'static> {
    TextInput::new(NAME)
        .placeholder("Your name")
        .blur(BlurPolicy::CommitAndValidate)
}

fn branch_input() -> TextInput<'static> {
    TextInput::new(BRANCH)
        .placeholder("Branch name")
        .blur(BlurPolicy::Commit)
}

fn fields_panel() -> Panel<'static> {
    Panel::new(CARD).kind(PanelKind::Card).title("Edit fields")
}

/// A pair of independent controlled fields, matching the legacy input page.
#[derive(Debug)]
pub(crate) struct InputsPage {
    name: String,
    branch: String,
    name_state: TextInputState,
    branch_state: TextInputState,
    last: &'static str,
}

impl InputsPage {
    pub(crate) fn new() -> Self {
        Self {
            name: String::from("operator"),
            branch: String::from("payments-gateway"),
            name_state: TextInputState::default(),
            branch_state: TextInputState::default(),
            last: "ready",
        }
    }
}

impl Default for InputsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for InputsPage {
    fn title(&self) -> &'static str {
        "Inputs"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let _ = fields_panel();
        let name = name_input().update(cx, &mut self.name_state, &mut self.name);
        if let Some(action) = name.action_ref() {
            self.last = match action {
                TextAction::Committed => "name committed",
                TextAction::Cancelled => "name reverted",
                TextAction::Changed => "name draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "name focus moved",
            };
        }
        response |= name.erase();
        let branch = branch_input().update(cx, &mut self.branch_state, &mut self.branch);
        if let Some(action) = branch.action_ref() {
            self.last = match action {
                TextAction::Committed => "branch committed",
                TextAction::Cancelled => "branch reverted",
                TextAction::Changed => "branch draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "branch focus moved",
            };
        }
        response |= branch.erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "controlled values · Enter commit · Esc cancel",
            |ui, body| {
                let regions = rows(body, 3);
                let fields = regions.first().copied().unwrap_or(body);
                fields_panel().draw(ui, fields, |ui, inner| {
                    let field_rows = rows(inner, 2);
                    name_input().value(&self.name).draw(
                        ui,
                        field_rows.first().copied().unwrap_or(inner),
                        &self.name_state,
                    );
                    branch_input().value(&self.branch).draw(
                        ui,
                        field_rows.get(1).copied().unwrap_or(inner),
                        &self.branch_state,
                    );
                });
                let facts = regions.get(1).copied().unwrap_or(body);
                let name_phase = if self.name_state.is_editing() {
                    "editing"
                } else {
                    "idle"
                };
                let branch_phase = if self.branch_state.is_editing() {
                    "editing"
                } else {
                    "idle"
                };
                let info = format!(
                    "name={} [{}] · branch={} [{}] · {}",
                    self.name, name_phase, self.branch, branch_phase, self.last
                );
                let _ = ui.paint_str(facts, &info, ui.surface_style());
                lines(
                    ui,
                    regions.get(2).copied().unwrap_or(body),
                    &[
                        "The draft lives in TextInputState until Enter commits it.",
                        "Esc cancels the draft without changing the controlled value.",
                    ],
                );
            },
        );
    }
}
