use termrock::{
    Action, ActionKey, Button, Cx, FieldError, FieldKind, FieldMut, FieldRef, FieldSpec, FormData,
    FormDialog, FormDialogAction, FormDialogState, GlyphRole, Id, Item, ItemKey, Panel, PanelKind,
    Part, Picker, PickerAction, PickerState, Rect, Response, Role, Secret, SecretPolicy, TextInput,
    Ui, Variant,
};
use termrock_fields::FocusSelect;

use super::{Page, PageStatus, PageUpdate, frame};

const OPEN_FORM: Id = Id::root("showcase_ui::pages::dialog_kinds::dialog_kinds.open_form");
const FORM: Id = Id::root("showcase_ui::pages::dialog_kinds::dialog_kinds.connection");
const NAME: Id = FORM.part(Part::BODY).part(Part::FIELD).index(0);
const DRIVER: Id = FORM.part(Part::BODY).part(Part::FIELD).index(1);
const HOST: Id = FORM.part(Part::BODY).part(Part::FIELD).index(2);
const PASSWORD: Id = FORM.part(Part::BODY).part(Part::FIELD).index(3);
const SSL: Id = FORM.part(Part::BODY).part(Part::FIELD).index(4);
const TUNNEL: Id = FORM.part(Part::BODY).part(Part::FIELD).index(5);
const KEY: Id = FORM.part(Part::BODY).part(Part::FIELD).index(6);
const PREVIEW: Id = FORM.part(Part::BODY).part(Part::FIELD).index(7);
const RESULT: Id = FORM.part(Part::BODY).part(Part::FIELD).index(8);
const TEST: ActionKey = ActionKey::custom("showcase.dialog_kinds.test_connection");
const PICKER: Id = Id::root("showcase_ui::pages::dialog_kinds::dialog_kinds.ssh_keys");

const DRIVERS: &[&str] = &["PostgreSQL", "MySQL", "SQLite"];
const SSL_MODES: &[&str] = &["Disable", "Require", "Verify full"];
const KEY_FILES: &[Item<'static>] = &[
    Item::new(ItemKey::index(0), "~/.ssh/id_ed25519").glyph("⚿"),
    Item::new(ItemKey::index(1), "~/.ssh/prod_rsa").glyph("⚿"),
    Item::new(ItemKey::index(2), "~/keys/staging.pem").glyph("⚿"),
];
const ACTIONS: &[Action<'static>] = &[
    Action::new(TEST, "Test connection").with_variant(Variant::SECONDARY),
    Action::new(ActionKey::SAVE, "Save").with_variant(Variant::PRIMARY),
];
const FIELDS: &[FieldSpec<'static>] = &[
    FieldSpec::new(
        NAME,
        "Name",
        FieldKind::Text(TextInput::new(NAME).focus_select(FocusSelect::SelectAll)),
    )
    .required(true),
    FieldSpec::new(
        DRIVER,
        "Driver",
        FieldKind::Select(termrock::Select::new(DRIVER)),
    ),
    FieldSpec::new(HOST, "Host", FieldKind::Text(TextInput::new(HOST))).required(true),
    FieldSpec::new(
        PASSWORD,
        "Password",
        FieldKind::Text(TextInput::new(PASSWORD).secret(SecretPolicy {
            mask: GlyphRole::SecretMask,
            synthetic_tail: 4,
        })),
    ),
    FieldSpec::new(
        SSL,
        "SSL mode",
        FieldKind::Radio(termrock::LabelRadio::new(SSL)),
    ),
    FieldSpec::new(
        TUNNEL,
        "",
        FieldKind::Check(termrock::Checkbox::new(
            TUNNEL,
            "Connect through an SSH tunnel",
        )),
    ),
    FieldSpec::new(
        KEY,
        "SSH key",
        FieldKind::Chooser(Button::new(KEY, "Choose…")),
    ),
    // FormData::Note currently requires borrowed row slices that cannot safely
    // represent this live preview. Keep preview data live through the shared
    // read-only text control until Form exposes an owned note-view contract.
    FieldSpec::new(
        PREVIEW,
        "Preview",
        FieldKind::Text(TextInput::new(PREVIEW).read_only(true)),
    ),
    FieldSpec::new(RESULT, "Result", FieldKind::Note),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TestResult {
    None,
    Connected,
    MissingHost,
}

const NO_RESULT: &[(&str, Role)] = &[];
const CONNECTED: &[(&str, Role)] = &[("✓ Connected in 42 ms", Role::Fg(termrock::FgStep::Primary))];
const MISSING_HOST: &[(&str, Role)] =
    &[("✗ Host is required", Role::Fg(termrock::FgStep::Primary))];

#[derive(Debug)]
struct ConnectionData {
    name: String,
    driver: usize,
    host: String,
    password: Secret,
    ssl: usize,
    tunnel: bool,
    key_file: String,
    preview: String,
    result: TestResult,
}

impl Default for ConnectionData {
    fn default() -> Self {
        let mut data = Self {
            name: String::from("orders-prod"),
            driver: 0,
            host: String::from("db.eu-west-1.internal"),
            password: Secret::new(String::from("hunter2-prod-7731")),
            ssl: 2,
            tunnel: false,
            key_file: String::from("~/.ssh/id_ed25519"),
            preview: String::new(),
            result: TestResult::None,
        };
        data.refresh_committed_preview();
        data
    }
}

impl ConnectionData {
    fn refresh_preview(&mut self, name: &str, host: &str) {
        let scheme = ["postgres", "mysql", "sqlite"]
            .get(self.driver)
            .copied()
            .unwrap_or("postgres");
        self.preview = format!("Preview  {scheme}://{host}/{name}");
    }

    fn refresh_committed_preview(&mut self) {
        let name = self.name.clone();
        let host = self.host.clone();
        self.refresh_preview(&name, &host);
    }

    fn result_rows(&self) -> &'static [(&'static str, Role)] {
        match self.result {
            TestResult::None => NO_RESULT,
            TestResult::Connected => CONNECTED,
            TestResult::MissingHost => MISSING_HOST,
        }
    }
}

impl FormData for ConnectionData {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            NAME => FieldRef::Text(&self.name),
            DRIVER => FieldRef::Choice(self.driver),
            HOST => FieldRef::Text(&self.host),
            PASSWORD => FieldRef::Secret(&self.password),
            SSL => FieldRef::Choice(self.ssl),
            TUNNEL => FieldRef::Flag(self.tunnel),
            KEY => FieldRef::Display {
                value: &self.key_file,
                detail: None,
            },
            PREVIEW => FieldRef::Text(&self.preview),
            RESULT => FieldRef::Note(self.result_rows()),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            NAME => FieldMut::Text(&mut self.name),
            DRIVER => FieldMut::Choice(&mut self.driver),
            HOST => FieldMut::Text(&mut self.host),
            PASSWORD => FieldMut::Secret(&mut self.password),
            SSL => FieldMut::Choice(&mut self.ssl),
            TUNNEL => FieldMut::Flag(&mut self.tunnel),
            _ => FieldMut::ReadOnly,
        }
    }

    fn options(&self, id: Id) -> &[&str] {
        match id {
            DRIVER => DRIVERS,
            SSL => SSL_MODES,
            _ => &[],
        }
    }

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        let options = match id {
            DRIVER => DRIVERS,
            SSL => SSL_MODES,
            _ => &[],
        };
        (self.value_mut(id), options)
    }

    fn visible(&self, id: Id) -> bool {
        id != KEY || self.tunnel
    }

    fn validate(&self, id: Id, value: FieldRef<'_>) -> Result<(), FieldError> {
        if id == NAME && matches!(value, FieldRef::Text(value) if value.trim().is_empty()) {
            return Err(FieldError::new("Connection name is required"));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DialogKindsPage {
    state: FormDialogState,
    data: ConnectionData,
    picker_state: PickerState,
    form_open: bool,
    picker_open: bool,
    last_result: Option<String>,
}

impl DialogKindsPage {
    pub fn new() -> Self {
        Self {
            state: FormDialogState::default(),
            data: ConnectionData::default(),
            picker_state: PickerState::default(),
            form_open: false,
            picker_open: false,
            last_result: None,
        }
    }

    fn form() -> FormDialog<'static> {
        FormDialog::new(FORM, "Add connection", FIELDS)
            .actions(ACTIONS)
            .submit(ActionKey::SAVE)
            .cancel_action(ActionKey::CANCEL)
            .width(62)
    }

    fn picker() -> Picker<'static, Item<'static>> {
        Picker::new(PICKER)
            .title("SSH key")
            .searchable(false)
            .width(48)
    }

    fn refresh_preview(&mut self) {
        let name = self
            .state
            .form()
            .editing_text(NAME)
            .unwrap_or(self.data.name.as_str())
            .to_owned();
        let host = self
            .state
            .form()
            .editing_text(HOST)
            .unwrap_or(self.data.host.as_str())
            .to_owned();
        self.data.refresh_preview(&name, &host);
    }

    fn close_form(&mut self, cx: &mut Cx<'_>, cancel: bool) {
        if cx.is_open(FORM) {
            cx.close_layer(FORM, None);
        }
        self.form_open = false;
        self.picker_open = false;
        if cancel {
            self.data = ConnectionData::default();
            self.last_result = Some(String::from("Form · Cancel"));
        }
    }
}

impl Default for DialogKindsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for DialogKindsPage {
    fn title(&self) -> &'static str {
        "Dialog kinds"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let open = Button::new(OPEN_FORM, "Add connection…").variant(Variant::PRIMARY);
        let open_response = open.update(cx);
        if open_response.activated() && !cx.is_open(FORM) {
            self.data = ConnectionData::default();
            self.state = FormDialogState::default();
            self.last_result = None;
            self.picker_open = false;
            self.form_open = true;
            self.picker_state = PickerState::default();
            cx.open_layer(FORM, Self::form().layer(cx, &self.data));
        }
        response |= open_response.erase();

        if cx.is_open(FORM) {
            if self.picker_open {
                let picker_response = Self::picker().update(cx, &mut self.picker_state, KEY_FILES);
                let picker_action = picker_response.action_ref().copied();
                response |= picker_response.erase();
                match picker_action {
                    Some(PickerAction::Chosen(key)) => {
                        if let Some(item) = KEY_FILES.iter().find(|item| item.key == key) {
                            self.data.key_file = item.label.to_owned();
                        }
                        cx.close_layer(PICKER, None);
                        self.picker_open = false;
                    }
                    Some(PickerAction::Back) => {
                        cx.close_layer(PICKER, None);
                        self.picker_open = false;
                    }
                    _ => {}
                }
            } else {
                let form_response = Self::form().update(cx, &mut self.state, &mut self.data);
                let action = form_response.action_ref().copied();
                response |= form_response.erase();
                match action {
                    Some(
                        FormDialogAction::Changed(_)
                        | FormDialogAction::Committed(_)
                        | FormDialogAction::Chose(_),
                    ) => self.refresh_preview(),
                    Some(FormDialogAction::Action(key)) if key == TEST => {
                        self.data.result = if self.data.host.trim().is_empty() {
                            TestResult::MissingHost
                        } else {
                            TestResult::Connected
                        };
                        response = response.repaint();
                    }
                    Some(FormDialogAction::Action(key)) if key == ActionKey::SAVE => {
                        self.data.refresh_committed_preview();
                        self.last_result = Some(format!("Form · Save · {}", self.data.name));
                    }
                    Some(FormDialogAction::Cancelled) => self.close_form(cx, true),
                    Some(FormDialogAction::Invalid(_)) => response = response.repaint(),
                    _ => {}
                }

                if matches!(action, Some(FormDialogAction::Chose(KEY))) {
                    self.picker_state = PickerState::default();
                    cx.open_layer(PICKER, Self::picker().layer(cx, KEY_FILES));
                    self.picker_open = true;
                }

                if matches!(action, Some(FormDialogAction::Action(key)) if key == ActionKey::SAVE)
                    || matches!(action, Some(FormDialogAction::Cancelled))
                {
                    self.close_form(cx, false);
                }
            }
        }

        let status = self
            .last_result
            .as_ref()
            .map(|message| PageStatus(message.clone()));
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Forms, info, choices and help",
            |ui, body| {
                let panel = Panel::new(OPEN_FORM)
                    .kind(PanelKind::Card)
                    .title("Open a dialog")
                    .meta("Enter · click");
                panel.draw(ui, body, |ui, inner| {
                    Button::new(OPEN_FORM, "Add connection…")
                        .variant(Variant::PRIMARY)
                        .draw(ui, inner);
                });
                if let Some(result) = &self.last_result {
                    ui.paint_str(
                        Rect {
                            y: body.y.saturating_add(5),
                            height: 1,
                            ..body
                        },
                        result,
                        ui.surface_style(),
                    );
                }
            },
        );
        if self.form_open {
            ui.layer(FORM, |ui, layer| {
                Self::form().draw(ui, layer, &self.state, &self.data);
            });
        }
        if self.picker_open {
            ui.layer(PICKER, |ui, layer| {
                Self::picker().draw(ui, layer, &self.picker_state, KEY_FILES);
            });
        }
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.picker_open {
            &[("↑↓", "Move"), ("Enter", "Choose"), ("Esc", "Back")]
        } else if self.form_open {
            &[("Tab", "Next field"), ("Enter", "Save"), ("Esc", "Cancel")]
        } else {
            &[("Enter", "Open")]
        }
    }
}
