use crate::domain::action::Action;
use crate::sim::task_args::{
    Invocation, InvocationPlan, TaskArgsAcceptError, TaskArgsProducer, TaskArgsSession,
};
use crate::sim::world::World;
use termrock::{
    Action as UiAction, ActionKey, Binding, Chord, Cx, EnterPolicy, FieldError, FieldKind,
    FieldMut, FieldRef, FieldSpec, FormData, FormDialog, FormDialogAction, FormDialogState,
    FormDialogSubmitPolicy, Id, Intent, KeyCode, KeyModifiers, Response, Select, TextInput, Ui,
};

const DIALOG: Id = Id::root("holla.task-args.dialog");
const COMMAND_SCOPE: Id = DIALOG.sub("commands");
const FILTER: Id = Id::root("holla.task-args.filter");
const PROFILE: Id = Id::root("holla.task-args.profile");
const FILTER_LABEL: &str = "Filterset expression";
const FILTER_HELP: &str =
    "Optional nextest filterset expression; passed through unchanged and not parsed here";
const PROFILE_LABEL: &str = "Profile";
const SCHEMA_VERSION: u16 = 1;
const PROFILE_OPTIONS: [&str; 2] = ["default", "ci"];
const COMMAND_BINDINGS: &[Binding<ActionKey>] = &[Binding {
    action: ActionKey::SAVE,
    chord: Some(Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL)),
    cmd: ActionKey::SAVE,
    label: "Review inputs",
    priority: 40,
    visible: false,
}];
const ACTIONS: &[UiAction<'static>] = &[
    UiAction::quiet(ActionKey::CANCEL, "Cancel"),
    UiAction::new(ActionKey::SAVE, "Review inputs")
        .chord(Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL)),
];
const FIELDS: &[FieldSpec<'static>] = &[
    FieldSpec::new(
        FILTER,
        FILTER_LABEL,
        FieldKind::Text(TextInput::new(FILTER).placeholder("test(tests::parser)")),
    )
    .help(FILTER_HELP),
    FieldSpec::new(
        PROFILE,
        PROFILE_LABEL,
        FieldKind::Select(Select::new(PROFILE)),
    )
    .required(true),
];

#[derive(Default)]
struct Values {
    filter: String,
    profile: usize,
}

impl FormData for Values {
    fn value(&self, id: Id) -> FieldRef<'_> {
        if id == FILTER {
            FieldRef::Text(&self.filter)
        } else if id == PROFILE {
            FieldRef::Choice(self.profile)
        } else {
            FieldRef::Display {
                value: "",
                detail: None,
            }
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        if id == FILTER {
            FieldMut::Text(&mut self.filter)
        } else if id == PROFILE {
            FieldMut::Choice(&mut self.profile)
        } else {
            FieldMut::ReadOnly
        }
    }

    fn options(&self, id: Id) -> &[&str] {
        if id == PROFILE { &PROFILE_OPTIONS } else { &[] }
    }

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        if id == FILTER {
            (FieldMut::Text(&mut self.filter), &[])
        } else if id == PROFILE {
            (FieldMut::Choice(&mut self.profile), &PROFILE_OPTIONS)
        } else {
            (FieldMut::ReadOnly, &[])
        }
    }
}

pub(crate) enum Event {
    Cancelled,
    Rejected(String),
    Accepted {
        producer: TaskArgsProducer,
        plan: Box<InvocationPlan>,
    },
}

pub(crate) struct TaskArgsEditor {
    action: Action,
    producer: TaskArgsProducer,
    session: TaskArgsSession,
    values: Values,
    state: FormDialogState,
}

impl TaskArgsEditor {
    pub(crate) fn new(
        action: Action,
        producer: TaskArgsProducer,
        session: TaskArgsSession,
    ) -> Result<Self, String> {
        let schema = producer.schema();
        let Some(filter) = schema.field("filter") else {
            return Err("Task input schema changed · review again".into());
        };
        let Some(profile) = schema.field("profile") else {
            return Err("Task input schema changed · review again".into());
        };
        let profile_choices_match = profile.choices().len() == PROFILE_OPTIONS.len()
            && profile
                .choices()
                .iter()
                .zip(PROFILE_OPTIONS)
                .all(|(actual, expected)| actual.as_str() == expected);
        if schema.version() != SCHEMA_VERSION
            || schema.fields().len() != 2
            || filter.label() != FILTER_LABEL
            || filter.help() != Some(FILTER_HELP)
            || filter.sensitivity() != crate::domain::args::InputSensitivity::Public
            || filter.required()
            || !filter.choices().is_empty()
            || profile.label() != PROFILE_LABEL
            || profile.sensitivity() != crate::domain::args::InputSensitivity::Public
            || !profile.required()
            || profile.default_value() != Some("default")
            || !profile_choices_match
        {
            return Err("Task input schema changed · review again".into());
        }
        let profile_value = profile
            .default_value()
            .and_then(|default| PROFILE_OPTIONS.iter().position(|choice| *choice == default))
            .ok_or_else(|| "Task input schema changed · review again".to_owned())?;
        let values = Values {
            filter: filter.default_value().unwrap_or_default().to_owned(),
            profile: profile_value,
        };
        Ok(Self {
            action,
            producer,
            session,
            values,
            state: FormDialogState::default(),
        })
    }

    pub(crate) fn open(&self, cx: &mut Cx<'_>) {
        cx.open_layer(DIALOG, props().layer(cx, &self.values));
        cx.focus(FILTER);
    }

    pub(crate) fn poll(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut state = std::mem::take(&mut self.state);
        let response = props().update(cx, &mut state, &mut self.values);
        self.state = state;
        response.erase()
    }

    pub(crate) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        world: &World,
    ) -> (Response<()>, Option<Event>) {
        let mut state = std::mem::take(&mut self.state);
        let mut form_response = props().update(cx, &mut state, &mut self.values);
        let mut action = form_response.take_action();
        let mut response = form_response.erase();
        let scoped_save = cx.intents(COMMAND_SCOPE).any(|intent| {
            matches!(intent, Intent::Binding(action)
                if Binding::command(COMMAND_BINDINGS, action) == Some(ActionKey::SAVE))
        });
        if scoped_save && action.is_none() {
            let mut submit_response = props().submit_current(cx, &mut state, &mut self.values);
            action = submit_response.take_action();
            response |= submit_response.erase();
        }
        self.state = state;
        let event = match action {
            Some(FormDialogAction::Cancelled) => {
                cx.close_layer(DIALOG, Some(ActionKey::CANCEL));
                Some(Event::Cancelled)
            }
            Some(FormDialogAction::Action(ActionKey::SAVE)) => self.accept(cx, world),
            Some(FormDialogAction::Invalid(id)) => {
                let message = "Enter a supported task input";
                self.state
                    .form_mut()
                    .set_error(id, Some(FieldError::new(message)));
                cx.focus(id);
                None
            }
            Some(
                FormDialogAction::Action(_)
                | FormDialogAction::Changed(_)
                | FormDialogAction::Committed(_)
                | FormDialogAction::Chose(_),
            )
            | None => None,
        };
        (response.erase(), event)
    }

    fn accept(&mut self, cx: &mut Cx<'_>, world: &World) -> Option<Event> {
        let Some(profile) = PROFILE_OPTIONS.get(self.values.profile).copied() else {
            self.state
                .form_mut()
                .set_error(PROFILE, Some(FieldError::new("Choose a supported profile")));
            cx.focus(PROFILE);
            return None;
        };
        let values = [
            ("filter", self.values.filter.as_str()),
            ("profile", profile),
        ];
        match self
            .session
            .accept(world, &self.action, &self.producer, &values)
        {
            Ok(plan) => {
                cx.close_layer(DIALOG, Some(ActionKey::SAVE));
                Some(Event::Accepted {
                    producer: self.producer.clone(),
                    plan: Box::new(plan),
                })
            }
            Err(TaskArgsAcceptError::Input(error)) => {
                let (id, message) = input_error_field(&error);
                self.state
                    .form_mut()
                    .set_error(id, Some(FieldError::new(message)));
                cx.focus(id);
                None
            }
            Err(TaskArgsAcceptError::Binding(error)) => {
                cx.close_layer(DIALOG, Some(ActionKey::SAVE));
                Some(Event::Rejected(error.to_string()))
            }
        }
    }

    pub(crate) fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(DIALOG, |ui, area| {
            ui.command_scope(COMMAND_SCOPE, COMMAND_BINDINGS, |ui| {
                props().draw(ui, area, &self.state, &self.values);
            });
        });
    }
}

fn input_error_field(error: &crate::domain::args::InputError) -> (Id, &'static str) {
    use crate::domain::args::InputError;
    match error {
        InputError::MissingRequired(field) | InputError::InvalidChoice(field)
            if field == "profile" =>
        {
            (PROFILE, "Choose a supported profile")
        }
        InputError::InvalidFieldId(_)
        | InputError::UnknownField(_)
        | InputError::DuplicateField(_)
        | InputError::MissingRequired(_)
        | InputError::InvalidChoice(_) => (FILTER, "Task input changed · review again"),
    }
}

fn props() -> FormDialog<'static> {
    FormDialog::new(DIALOG, "Task inputs", FIELDS)
        .description("Review the explicit nextest filterset and profile request")
        .submit(ActionKey::SAVE)
        .cancel_action(ActionKey::CANCEL)
        .actions(ACTIONS)
        .enter_policy(EnterPolicy::Never)
        .submit_policy(FormDialogSubmitPolicy::KeepOpen)
}

pub(crate) fn quote_value(value: &str) -> String {
    quote(value)
}

pub(crate) fn format_invocation(invocation: &Invocation) -> String {
    format_invocation_lines(invocation).join(" · ")
}

pub(crate) fn format_invocation_lines(invocation: &Invocation) -> Vec<String> {
    let mut lines = vec![format!("program {}", quote(invocation.program()))];
    lines.extend(
        invocation
            .argv()
            .iter()
            .enumerate()
            .map(|(index, operand)| format!("argv[{index}] {}", quote(operand))),
    );
    lines.push(format!("target {}", quote(invocation.target())));
    lines.push(format!("workdir {}", quote(invocation.workdir())));
    lines
}

fn quote(value: &str) -> String {
    use std::fmt::Write as _;

    let mut escaped = String::with_capacity(value.len().saturating_add(2));
    escaped.push('\'');
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            character if character.is_control() => {
                let _ = write!(escaped, "\\u{{{:x}}}", u32::from(character));
            }
            character => escaped.push(character),
        }
    }
    escaped.push('\'');
    escaped
}
