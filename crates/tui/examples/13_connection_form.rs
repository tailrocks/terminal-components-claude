//! Fifteen-field connection form from `COMPONENT_ARCHITECTURE.md` §17 example 13.

use junie_tui::{
    Action, ActionKey, App, Checkbox, Chord, Cx, EnterPolicy, FieldError, FieldKind, FieldMut,
    FieldRef, FieldSpan, FieldSpec, Form, FormAction, FormData, FormState, GroupKey, Id, KeyCode,
    KeyModifiers, RadioGroup, Response, Secret, SecretPolicy, Select, TextArea, TextInput, Toggle,
    Ui, id,
};

const FORM: Id = id!("connections.form");
const NAME: Id = id!("connections.form.name");
const ENGINE: Id = id!("connections.form.engine");
const HOST: Id = id!("connections.form.host");
const PORT: Id = id!("connections.form.port");
const DB: Id = id!("connections.form.db");
const USER: Id = id!("connections.form.user");
const PW: Id = id!("connections.form.pw");
const ASKPW: Id = id!("connections.form.askpw");
const ENV: Id = id!("connections.form.env");
const GROUP: Id = id!("connections.form.group");
const SAFE: Id = id!("connections.form.safe");
const SSL: Id = id!("connections.form.ssl");
const SSH: Id = id!("connections.form.ssh");
const SSHH: Id = id!("connections.form.sshhost");
const START: Id = id!("connections.form.startup");
const BASIC: GroupKey = GroupKey::custom("basic");
const ADV: GroupKey = GroupKey::custom("advanced");
const TEST: ActionKey = ActionKey::custom("test");
const SAVE_CONNECT: ActionKey = ActionKey::custom("save+connect");

const ENGINES: &[&str] = &["PostgreSQL", "MySQL", "SQLite"];
const ENVS: &[&str] = &["Development", "Staging", "Production"];
const GROUPS: &[&str] = &["Default", "Analytics", "Operations"];
const MODES: &[&str] = &["Off", "Read only", "Confirm writes"];

fn port_rule(value: &str) -> Result<(), FieldError> {
    if value.is_empty() || value.parse::<u16>().is_ok() {
        Ok(())
    } else {
        Err(FieldError::coded("Enter a valid port", "port"))
    }
}

fn default_port(engine: usize) -> &'static str {
    match engine {
        0 => "5432",
        1 => "3306",
        _ => "",
    }
}

fn conn_fields<'a>() -> [FieldSpec<'a>; 15] {
    [
        FieldSpec::new(NAME, "Name", FieldKind::Text(TextInput::new(NAME)))
            .required(true)
            .group(BASIC),
        FieldSpec::new(ENGINE, "Engine", FieldKind::Select(Select::new(ENGINE))).group(BASIC),
        FieldSpec::new(
            HOST,
            "Host",
            FieldKind::Text(TextInput::new(HOST).placeholder("localhost")),
        )
        .help("Blank: driver default")
        .span(FieldSpan::Half)
        .group(BASIC),
        FieldSpec::new(
            PORT,
            "Port",
            FieldKind::Text(TextInput::new(PORT).validate(&port_rule)),
        )
        .span(FieldSpan::Half)
        .group(BASIC),
        FieldSpec::new(DB, "Database", FieldKind::Text(TextInput::new(DB)))
            .help("Required for PostgreSQL")
            .group(BASIC),
        FieldSpec::new(USER, "Username", FieldKind::Text(TextInput::new(USER))).group(BASIC),
        FieldSpec::new(
            PW,
            "Password",
            FieldKind::Text(TextInput::new(PW).secret(SecretPolicy::default())),
        )
        .help("Never written to connections.json")
        .group(BASIC),
        FieldSpec::new(
            ASKPW,
            "",
            FieldKind::Check(Checkbox::new(ASKPW, "Prompt for password on connect")),
        )
        .plain(true)
        .group(BASIC),
        FieldSpec::new(ENV, "Environment", FieldKind::Radio(RadioGroup::new(ENV))).group(BASIC),
        FieldSpec::new(GROUP, "Group", FieldKind::Select(Select::new(GROUP))).group(BASIC),
        FieldSpec::new(SAFE, "Safe Mode", FieldKind::Radio(RadioGroup::new(SAFE))).group(BASIC),
        FieldSpec::new(
            SSL,
            "",
            FieldKind::Toggle(Toggle::new(SSL, "Use SSL / TLS")),
        )
        .plain(true)
        .group(ADV),
        FieldSpec::new(SSH, "", FieldKind::Toggle(Toggle::new(SSH, "SSH tunnel")))
            .plain(true)
            .group(ADV),
        FieldSpec::new(
            SSHH,
            "SSH host",
            FieldKind::Text(TextInput::new(SSHH).placeholder("bastion.example.com")),
        )
        .group(ADV),
        FieldSpec::new(
            START,
            "Startup commands",
            FieldKind::Area(TextArea::new(START, 3)),
        )
        .help("Run after every connect, one per line")
        .group(ADV),
    ]
}

fn conn_actions() -> [Action<'static>; 4] {
    [
        Action::quiet(TEST, "Test connection"),
        Action::new(ActionKey::CANCEL, "Cancel"),
        Action::new(ActionKey::SAVE, "Save")
            .chord(Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL)),
        Action::new(SAVE_CONNECT, "Save & connect"),
    ]
}

#[derive(Default)]
struct ConnDraft {
    name: String,
    engine: usize,
    host: String,
    port: String,
    database: String,
    username: String,
    password: Secret,
    ask_password: bool,
    environment: usize,
    group: usize,
    safe_mode: usize,
    ssl: bool,
    ssh: bool,
    ssh_host: String,
    startup: String,
}

impl ConnDraft {
    fn option_table(id: Id) -> &'static [&'static str] {
        match id {
            ENGINE => ENGINES,
            ENV => ENVS,
            GROUP => GROUPS,
            SAFE => MODES,
            _ => &[],
        }
    }
}

impl FormData for ConnDraft {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            NAME => FieldRef::Text(&self.name),
            ENGINE => FieldRef::Choice(self.engine),
            HOST => FieldRef::Text(&self.host),
            PORT => FieldRef::Text(&self.port),
            DB => FieldRef::Text(&self.database),
            USER => FieldRef::Text(&self.username),
            PW => FieldRef::Secret(&self.password),
            ASKPW => FieldRef::Flag(self.ask_password),
            ENV => FieldRef::Choice(self.environment),
            GROUP => FieldRef::Choice(self.group),
            SAFE => FieldRef::Choice(self.safe_mode),
            SSL => FieldRef::Flag(self.ssl),
            SSH => FieldRef::Flag(self.ssh),
            SSHH => FieldRef::Text(&self.ssh_host),
            START => FieldRef::Text(&self.startup),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            NAME => FieldMut::Text(&mut self.name),
            ENGINE => FieldMut::Choice(&mut self.engine),
            HOST => FieldMut::Text(&mut self.host),
            PORT => FieldMut::Text(&mut self.port),
            DB => FieldMut::Text(&mut self.database),
            USER => FieldMut::Text(&mut self.username),
            PW => FieldMut::Secret(&mut self.password),
            ASKPW => FieldMut::Flag(&mut self.ask_password),
            ENV => FieldMut::Choice(&mut self.environment),
            GROUP => FieldMut::Choice(&mut self.group),
            SAFE => FieldMut::Choice(&mut self.safe_mode),
            SSL => FieldMut::Flag(&mut self.ssl),
            SSH => FieldMut::Flag(&mut self.ssh),
            SSHH => FieldMut::Text(&mut self.ssh_host),
            START => FieldMut::Text(&mut self.startup),
            _ => FieldMut::ReadOnly,
        }
    }

    fn options(&self, id: Id) -> &[&str] {
        Self::option_table(id)
    }

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        (self.value_mut(id), Self::option_table(id))
    }

    fn visible(&self, id: Id) -> bool {
        id != SSHH || self.ssh
    }

    fn validate(&self, id: Id, value: FieldRef<'_>) -> Result<(), FieldError> {
        match (id, value) {
            (NAME, FieldRef::Text(value)) if value.trim().is_empty() => {
                Err(FieldError::coded("Required", "required"))
            }
            (PORT, FieldRef::Text(value)) => port_rule(value),
            _ => Ok(()),
        }
    }

    fn validate_all(&self) -> Result<(), (Id, FieldError)> {
        if self.engine == 0 && self.database.trim().is_empty() {
            Err((DB, FieldError::new("PostgreSQL needs a database")))
        } else {
            Ok(())
        }
    }
}

struct ConnScreen {
    draft: ConnDraft,
    form: FormState,
    tab: GroupKey,
}

fn form_props<'a>(
    tab: GroupKey,
    fields: &'a [FieldSpec<'a>],
    actions: &'a [Action<'a>],
) -> Form<'a> {
    Form::new(FORM, fields)
        .actions(actions)
        .submit(ActionKey::SAVE)
        .enter(EnterPolicy::SubmitsWhenIdle)
        .columns(2)
        .group(tab)
}

impl ConnScreen {
    fn save(_connect: bool) {}

    fn begin_test() {}

    fn close(_cx: &mut Cx<'_>) {}
}

impl App for ConnScreen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let fields = conn_fields();
        let actions = conn_actions();
        form_props(self.tab, &fields, &actions)
            .update(cx, &mut self.form, &mut self.draft)
            .on_action(|action| match action {
                FormAction::Committed(ENGINE) => {
                    self.draft.port.clear();
                    self.draft.port.push_str(default_port(self.draft.engine));
                }
                FormAction::Action(ActionKey::SAVE) => Self::save(false),
                FormAction::Action(SAVE_CONNECT) => Self::save(true),
                FormAction::Action(TEST) => Self::begin_test(),
                FormAction::Action(ActionKey::CANCEL) => Self::close(cx),
                FormAction::Changed(_)
                | FormAction::Committed(_)
                | FormAction::Chose(_)
                | FormAction::Action(_)
                | FormAction::Invalid(_) => {}
            })
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let fields = conn_fields();
        let actions = conn_actions();
        form_props(self.tab, &fields, &actions).draw(ui, ui.full(), &self.form, &self.draft);
    }
}

fn main() {
    let _screen = ConnScreen {
        draft: ConnDraft::default(),
        form: FormState::default(),
        tab: BASIC,
    };
}
