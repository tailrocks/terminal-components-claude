//! Form-like composition with required-field validation.

use tui_next::{
    ActionKey, Button, Checkbox, Cx, Id, ItemKey, Rect, Response, Select, SelectState, TextArea,
    TextAreaState, TextInput, TextInputState, Ui, Variant, id, layout,
};

use super::{Page, frame, lines, rows};

/// Application-level submit chord consumed by the shell and forwarded here.
pub(crate) const SUBMIT: ActionKey = ActionKey::custom("showcase.form.submit");

const SUMMARY: Id = id!("forms.summary");
const DETAILS: Id = id!("forms.details");
const PRIORITY: Id = id!("forms.priority");
const CONFIRM: Id = id!("forms.confirm");
const SAVE: Id = id!("forms.save");

/// A composed form owns each field's controlled value and validation state.
#[derive(Debug)]
pub(crate) struct FormsPage {
    summary: String,
    details: String,
    summary_state: TextInputState,
    details_state: TextAreaState,
    priority: SelectState,
    confirm: bool,
    error: Option<&'static str>,
    submitted: bool,
}

impl FormsPage {
    pub(crate) fn new() -> Self {
        let mut priority = SelectState::default();
        priority.set_value(Some(ItemKey::index(0)));
        Self {
            summary: String::new(),
            details: String::from("Describe the change and its rollback plan."),
            summary_state: TextInputState::default(),
            details_state: TextAreaState::default(),
            priority,
            confirm: false,
            error: None,
            submitted: false,
        }
    }

    fn summary() -> TextInput<'static> {
        TextInput::new(SUMMARY).placeholder("Short imperative summary")
    }

    fn details() -> TextArea<'static> {
        TextArea::new(DETAILS, 4).placeholder("Details")
    }

    fn priority() -> Select<'static, &'static str> {
        Select::new(PRIORITY).placeholder("Priority")
    }

    fn validate(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.summary.trim().is_empty() {
            self.error = Some("Required: summary");
            cx.focus(SUMMARY);
            return Response::changed();
        }
        if self.details.trim().is_empty() {
            self.error = Some("Required: details");
            cx.focus(DETAILS);
            return Response::changed();
        }
        self.error = None;
        self.submitted = true;
        Response::changed()
    }
}

impl Default for FormsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for FormsPage {
    fn title(&self) -> &'static str {
        "Forms"
    }

    fn command(&mut self, cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        if action == SUBMIT {
            self.validate(cx)
        } else {
            Response::ignored()
        }
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        result |= Self::summary()
            .update(cx, &mut self.summary_state, &mut self.summary)
            .erase();
        result |= Self::details()
            .update(cx, &mut self.details_state, &mut self.details)
            .erase();
        result |= Self::priority()
            .update(cx, &mut self.priority, &["Normal", "High", "Urgent"])
            .erase();
        result |= Checkbox::new(CONFIRM, "I reviewed the rollback plan")
            .update(cx, &mut self.confirm)
            .erase();
        let save = Button::new(SAVE, "Create task")
            .variant(Variant::PRIMARY)
            .disabled(!self.confirm)
            .update(cx);
        if save.activated() {
            result |= self.validate(cx);
        }
        result |= save.erase();
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "controlled fields · Ctrl+S submit · validation",
            |ui, body| {
                let (left, right) = layout::split_h(body, body.width / 2);
                let left_rows = rows(left, 4);
                Self::summary().value(&self.summary).draw(
                    ui,
                    left_rows.first().copied().unwrap_or(left),
                    &self.summary_state,
                );
                Self::details().value(&self.details).draw(
                    ui,
                    left_rows.get(1).copied().unwrap_or(left),
                    &self.details_state,
                );
                Self::priority().draw(
                    ui,
                    left_rows.get(2).copied().unwrap_or(left),
                    &self.priority,
                    &["Normal", "High", "Urgent"],
                );
                Checkbox::new(CONFIRM, "I reviewed the rollback plan")
                    .checked(self.confirm)
                    .draw(ui, left_rows.get(3).copied().unwrap_or(left));
                let right_rows = rows(right, 4);
                Button::new(SAVE, "Create task")
                    .variant(Variant::PRIMARY)
                    .disabled(!self.confirm)
                    .draw(ui, right_rows.first().copied().unwrap_or(right));
                let status = self.error.unwrap_or(if self.submitted {
                    "Creating task"
                } else {
                    "Required fields are marked before submit"
                });
                let _ = ui.paint_str(
                    right_rows.get(1).copied().unwrap_or(right),
                    status,
                    ui.surface_style(),
                );
                lines(
                    ui,
                    right_rows.get(2).copied().unwrap_or(right),
                    &[
                        "Summary is the first validation target.",
                        "Select commits its value only after a choice.",
                    ],
                );
            },
        );
    }
}
