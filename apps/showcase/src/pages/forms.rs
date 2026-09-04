//! Form-like composition with required-field validation.

use junie_tui::{
    ActionKey, Button, Checkbox, Cx, Field, FieldKind, FieldMut, FieldRef, FieldSpec, Form,
    FormData, FormState, Id, ItemKey, Rect, Response, Role, Select, SelectState, TextArea,
    TextAreaState, TextInput, TextInputState, Ui, Variant, Wizard, WizardState, WizardStep, id,
    layout,
};

use super::{Page, frame, lines, rows};

/// Application-level submit chord consumed by the shell and forwarded here.
pub(crate) const SUBMIT: ActionKey = ActionKey::custom("showcase.form.submit");

const SUMMARY: Id = id!("forms.summary");
const DETAILS: Id = id!("forms.details");
const PRIORITY: Id = id!("forms.priority");
const CONFIRM: Id = id!("forms.confirm");
const SAVE: Id = id!("forms.save");
const PUBLIC_FORM: Id = id!("forms.public");
const PUBLIC_NOTE: Id = id!("forms.public.note");
const FIELD: Id = id!("forms.field");
const WIZARD: Id = id!("forms.wizard");
const NOTES: &[(&str, Role)] = &[("Values stay with the application owner.", Role::Info)];
const PUBLIC_FIELDS: &[FieldSpec<'static>] = &[FieldSpec::new(
    PUBLIC_NOTE,
    "FormData note",
    FieldKind::Note,
)];
const WIZARD_STEPS: &[WizardStep<'static>] = &[
    WizardStep::new(ItemKey::Num(201), "Draft"),
    WizardStep::new(ItemKey::Num(202), "Review"),
    WizardStep::new(ItemKey::Num(203), "Done"),
];

#[derive(Debug, Default)]
struct PublicFormData;

impl FormData for PublicFormData {
    fn value(&self, _id: Id) -> FieldRef<'_> {
        FieldRef::Note(NOTES)
    }

    fn value_mut(&mut self, _id: Id) -> FieldMut<'_> {
        FieldMut::ReadOnly
    }
}

fn public_form() -> Form<'static> {
    Form::new(PUBLIC_FORM, PUBLIC_FIELDS)
}

fn api_field() -> Field<'static, Checkbox<'static>> {
    Field::new(
        "Public field",
        Checkbox::new(FIELD, "draw-time chrome").disabled(true),
    )
    .help("Field owns labels; the control owns state.")
}

fn wizard() -> Wizard<'static> {
    Wizard::new(WIZARD, WIZARD_STEPS)
}

/// A composed form owns each field's controlled value and validation state.
#[derive(Debug)]
pub(crate) struct FormsPage {
    summary: String,
    details: String,
    summary_state: TextInputState,
    details_state: TextAreaState,
    priority: SelectState,
    public_form: FormState,
    wizard: WizardState,
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
            public_form: FormState::default(),
            wizard: WizardState::default(),
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

    fn confirmation() -> Checkbox<'static> {
        Checkbox::new(CONFIRM, "I reviewed the rollback plan")
    }

    fn save_button(confirmed: bool) -> Button<'static> {
        Button::new(SAVE, "Create task")
            .variant(Variant::PRIMARY)
            .disabled(!confirmed)
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
        result |= Self::confirmation().update(cx, &mut self.confirm).erase();
        let save = Self::save_button(self.confirm).update(cx);
        if save.activated() {
            result |= self.validate(cx);
        }
        result |= save.erase();
        let mut data = PublicFormData;
        result |= public_form()
            .update(cx, &mut self.public_form, &mut data)
            .erase();
        result |= wizard().update(cx, &mut self.wizard).erase();
        let _ = api_field();
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
                Self::confirmation()
                    .checked(self.confirm)
                    .draw(ui, left_rows.get(3).copied().unwrap_or(left));
                let right_rows = rows(right, 4);
                Self::save_button(self.confirm)
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
                if let Some(area) = right_rows.get(3).copied() {
                    let (form_area, field_area) = layout::split_v(area, 4);
                    let data = PublicFormData;
                    public_form().draw(ui, form_area, &self.public_form, &data);
                    let (field_area, wizard_area) = layout::split_v(field_area, 1);
                    api_field().draw(ui, field_area, &());
                    wizard().draw(ui, wizard_area, &self.wizard);
                }
            },
        );
    }
}
