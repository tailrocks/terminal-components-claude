use crate::domain::action::Action;
use crate::sim::port_args::{
    PortArgsAcceptError, PortArgsProducer, PortArgsSession, PortInvocationPlan,
};
use crate::sim::world::World;
use termrock::{
    Action as UiAction, ActionKey, Binding, Chord, Cx, EnterPolicy, FieldError, FieldKind,
    FieldMut, FieldRef, FieldSpec, FormData, FormDialog, FormDialogAction, FormDialogState,
    FormDialogSubmitPolicy, Id, Intent, KeyCode, KeyModifiers, LayerOpening, Response, TextInput,
    Ui,
};

pub(crate) const DIALOG: Id = Id::root("holla.port-args.dialog");
const COMMAND_SCOPE: Id = DIALOG.sub("commands");
pub(crate) const PORT: Id = Id::root("holla.port-args.port");
const PORT_LABEL: &str = "TCP port";
const PORT_HELP: &str =
    "Required text; malformed non-empty input preserves the historical fallback to 5173";
const SCHEMA_VERSION: u16 = 1;
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
const FIELDS: &[FieldSpec<'static>] = &[FieldSpec::new(
    PORT,
    PORT_LABEL,
    FieldKind::Text(TextInput::new(PORT).placeholder("5173")),
)
.help(PORT_HELP)
.required(true)];

#[derive(Default)]
struct Values {
    port: String,
}

impl FormData for Values {
    fn value(&self, id: Id) -> FieldRef<'_> {
        if id == PORT {
            FieldRef::Text(&self.port)
        } else {
            FieldRef::Display {
                value: "",
                detail: None,
            }
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        if id == PORT {
            FieldMut::Text(&mut self.port)
        } else {
            FieldMut::ReadOnly
        }
    }

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        if id == PORT {
            (FieldMut::Text(&mut self.port), &[])
        } else {
            (FieldMut::ReadOnly, &[])
        }
    }

    fn validate(&self, id: Id, value: FieldRef<'_>) -> Result<(), FieldError> {
        if id == PORT && matches!(value, FieldRef::Text(value) if value.trim().is_empty()) {
            Err(FieldError::new("Enter a non-empty TCP port"))
        } else {
            Ok(())
        }
    }
}

pub(crate) enum Event {
    Cancelled,
    Rejected(String),
    Accepted {
        producer: PortArgsProducer,
        plan: Box<PortInvocationPlan>,
    },
}

pub(crate) struct PortArgsEditor {
    action: Action,
    producer: PortArgsProducer,
    session: Option<PortArgsSession>,
    values: Values,
    state: FormDialogState,
}

impl PortArgsEditor {
    pub(crate) fn new(
        action: Action,
        producer: PortArgsProducer,
        session: PortArgsSession,
    ) -> Result<Self, String> {
        let schema = producer.schema();
        let Some(port) = schema.field("port") else {
            return Err("Port input schema changed · review again".into());
        };
        if schema.version() != SCHEMA_VERSION
            || schema.fields().len() != 1
            || port.label() != PORT_LABEL
            || port.help() != Some(PORT_HELP)
            || port.sensitivity() != crate::domain::args::InputSensitivity::Public
            || !port.required()
            || !port.choices().is_empty()
            || port.default_value() != Some("5173")
        {
            return Err("Port input schema changed · review again".into());
        }
        let initial_port = port.default_value().unwrap_or_default().to_owned();
        Ok(Self {
            action,
            producer,
            session: Some(session),
            values: Values { port: initial_port },
            state: FormDialogState::default(),
        })
    }

    pub(crate) fn open(&self, cx: &mut Cx<'_>) -> Option<LayerOpening> {
        let opening = cx.open_layer_with_token(DIALOG, props().layer(cx, &self.values))?;
        cx.focus(PORT);
        Some(opening)
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
        let Some(session) = self.session.as_mut() else {
            cx.close_layer(DIALOG, Some(ActionKey::SAVE));
            return Some(Event::Rejected("Port input review already consumed".into()));
        };
        let result = session.accept(
            world,
            &self.action,
            &self.producer,
            &[("port", self.values.port.as_str())],
        );
        match result {
            Ok(plan) => {
                self.session.take();
                cx.close_layer(DIALOG, Some(ActionKey::SAVE));
                Some(Event::Accepted {
                    producer: self.producer.clone(),
                    plan: Box::new(plan),
                })
            }
            Err(PortArgsAcceptError::Input(error)) => {
                let message = match error {
                    crate::domain::args::InputError::MissingRequired(_) => {
                        "Enter a non-empty TCP port"
                    }
                    crate::domain::args::InputError::InvalidFieldId(_)
                    | crate::domain::args::InputError::UnknownField(_)
                    | crate::domain::args::InputError::DuplicateField(_)
                    | crate::domain::args::InputError::InvalidChoice(_) => {
                        "Port input changed · review again"
                    }
                };
                self.state
                    .form_mut()
                    .set_error(PORT, Some(FieldError::new(message)));
                cx.focus(PORT);
                None
            }
            Err(PortArgsAcceptError::Binding(error)) => {
                self.session.take();
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

fn props() -> FormDialog<'static> {
    FormDialog::new(DIALOG, "Port input", FIELDS)
        .description("Choose the port for a structured simulated lookup")
        .submit(ActionKey::SAVE)
        .cancel_action(ActionKey::CANCEL)
        .actions(ACTIONS)
        .enter_policy(EnterPolicy::Never)
        .submit_policy(FormDialogSubmitPolicy::KeepOpen)
}
