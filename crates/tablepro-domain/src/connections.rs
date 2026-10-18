#![doc = r" Connection list and the app-owned 15-field connection draft."]
#![allow(
    missing_docs,
    reason = "items promoted from pub(crate) during the crate split keep their original undocumented fixtures"
)]

use termrock::{
    Action, ActionKey, Checkbox, Chord, FieldKind, FieldMut, FieldRef, FieldSpan, FieldSpec,
    FormData, Id, RadioGroup, Secret, SecretPolicy, Select, TextArea, TextInput, Toggle,
};

use tablepro_demo::db::{self, ConnectOutcome, Connection, Engine, Environment, SafeMode};

#[doc = r" Connection form id."]
pub const FORM: Id = Id::root("tablepro.connections.form");

#[doc = r" Stable form field ids."]
pub mod field {
    use termrock::Id;

    #[doc = r" Connection name field."]
    pub const NAME: Id = Id::root("tablepro.connections.form.name");
    #[doc = r" Database engine field."]
    pub const ENGINE: Id = Id::root("tablepro.connections.form.engine");
    #[doc = r" Host name field."]
    pub const HOST: Id = Id::root("tablepro.connections.form.host");
    #[doc = r" Port field."]
    pub const PORT: Id = Id::root("tablepro.connections.form.port");
    #[doc = r" Database name field."]
    pub const DATABASE: Id = Id::root("tablepro.connections.form.database");
    #[doc = r" User name field."]
    pub const USER: Id = Id::root("tablepro.connections.form.user");
    #[doc = r" Password field."]
    pub const PASSWORD: Id = Id::root("tablepro.connections.form.password");
    #[doc = r" Prompt-for-password field."]
    pub const ASK_PASSWORD: Id = Id::root("tablepro.connections.form.ask-password");
    #[doc = r" Environment field."]
    pub const ENVIRONMENT: Id = Id::root("tablepro.connections.form.environment");
    #[doc = r" Connection group field."]
    pub const GROUP: Id = Id::root("tablepro.connections.form.group");
    #[doc = r" Safe-mode field."]
    pub const SAFE_MODE: Id = Id::root("tablepro.connections.form.safe-mode");
    #[doc = r" TLS toggle field."]
    pub const SSL: Id = Id::root("tablepro.connections.form.ssl");
    #[doc = r" SSH tunnel toggle field."]
    pub const SSH: Id = Id::root("tablepro.connections.form.ssh");
    #[doc = r" SSH host field."]
    pub const SSH_HOST: Id = Id::root("tablepro.connections.form.ssh-host");
    #[doc = r" Startup command field."]
    pub const STARTUP: Id = Id::root("tablepro.connections.form.startup");
}

#[doc = r" Test action."]
pub const TEST: ActionKey = ActionKey::application("tablepro.connections.test");
#[doc = r" Save-and-connect action."]
pub const SAVE_CONNECT: ActionKey = ActionKey::application("tablepro.connections.save-connect");

#[doc = r" Engine option labels."]
pub const ENGINES: &[&str] = &["PostgreSQL", "MySQL", "SQLite"];
#[doc = r" Environment option labels."]
pub const ENVIRONMENTS: &[&str] = &["Local", "Development", "Staging", "Production"];
#[doc = r" Group option labels."]
pub const GROUPS: &[&str] = &["Personal", "Acme"];
#[doc = r" Safe-mode option labels."]
pub const SAFE_MODES: &[&str] = &[
    "Silent",
    "Alert",
    "Alert (Full)",
    "Safe Mode",
    "Safe Mode (Full)",
    "Read-Only",
];

#[doc = r" Default port for an engine choice."]
pub const fn default_port(engine: usize) -> &'static str {
    match engine {
        0 => "5432",
        1 => "3306",
        _ => "",
    }
}

fn valid_port(value: &str) -> Result<(), termrock::FieldError> {
    if value.is_empty() || value.parse::<u16>().is_ok() {
        Ok(())
    } else {
        Err(termrock::FieldError::coded("Enter a valid port", "port"))
    }
}

#[doc = r" The public `Form` declaration for the complete connection editor."]
pub fn form_fields() -> [FieldSpec<'static>; 15] {
    use field::{
        ASK_PASSWORD, DATABASE, ENGINE, ENVIRONMENT, GROUP, HOST, NAME, PASSWORD, PORT, SAFE_MODE,
        SSH, SSH_HOST, SSL, STARTUP, USER,
    };
    [
        FieldSpec::new(NAME, "Name", FieldKind::Text(TextInput::new(NAME))).required(true),
        FieldSpec::new(ENGINE, "Engine", FieldKind::Select(Select::new(ENGINE))),
        FieldSpec::new(
            HOST,
            "Host",
            FieldKind::Text(TextInput::new(HOST).placeholder("localhost")),
        )
        .span(FieldSpan::Half),
        FieldSpec::new(
            PORT,
            "Port",
            FieldKind::Text(TextInput::new(PORT).validate(&valid_port)),
        )
        .span(FieldSpan::Half),
        FieldSpec::new(
            DATABASE,
            "Database",
            FieldKind::Text(TextInput::new(DATABASE)),
        ),
        FieldSpec::new(USER, "Username", FieldKind::Text(TextInput::new(USER))),
        FieldSpec::new(
            PASSWORD,
            "Password",
            FieldKind::Text(TextInput::new(PASSWORD).secret(SecretPolicy::default())),
        )
        .help("Never written to connections.json"),
        FieldSpec::new(
            ASK_PASSWORD,
            "",
            FieldKind::Check(Checkbox::new(
                ASK_PASSWORD,
                "Prompt for password on connect",
            )),
        )
        .plain(true),
        FieldSpec::new(
            ENVIRONMENT,
            "Environment",
            FieldKind::Radio(RadioGroup::new(ENVIRONMENT)),
        ),
        FieldSpec::new(GROUP, "Group", FieldKind::Select(Select::new(GROUP))),
        FieldSpec::new(
            SAFE_MODE,
            "Safe Mode",
            FieldKind::Radio(RadioGroup::new(SAFE_MODE)),
        ),
        FieldSpec::new(
            SSL,
            "",
            FieldKind::Toggle(Toggle::new(SSL, "Use SSL / TLS")),
        )
        .plain(true),
        FieldSpec::new(SSH, "", FieldKind::Toggle(Toggle::new(SSH, "SSH tunnel"))).plain(true),
        FieldSpec::new(
            SSH_HOST,
            "SSH host",
            FieldKind::Text(TextInput::new(SSH_HOST).placeholder("bastion.example.com")),
        ),
        FieldSpec::new(
            STARTUP,
            "Startup commands",
            FieldKind::Area(TextArea::new(STARTUP, 3)),
        )
        .help("Run after every connect, one per line"),
    ]
}

#[doc = r" The public `Form` action row."]
pub fn form_actions() -> [Action<'static>; 4] {
    [
        Action::quiet(TEST, "Test connection"),
        Action::new(ActionKey::CANCEL, "Cancel"),
        Action::new(ActionKey::SAVE, "Save").chord(Chord::with(
            termrock::KeyCode::Char('s'),
            termrock::KeyModifiers::CONTROL,
        )),
        Action::new(SAVE_CONNECT, "Save & connect"),
    ]
}

#[doc = r" Controlled connection form data. Password storage is a `Secret`, so this"]
#[doc = r" type intentionally has no `Clone`, `Eq`, or string-producing debug path."]
#[derive(Debug, Default)]
pub struct ConnectionDraft {
    #[doc = r" Display name."]
    pub name: String,
    #[doc = r" Engine option index."]
    pub engine: usize,
    #[doc = r" Host name."]
    pub host: String,
    #[doc = r" Port text."]
    pub port: String,
    #[doc = r" Database name."]
    pub database: String,
    #[doc = r" User name."]
    pub user: String,
    #[doc = r" Password."]
    pub password: Secret,
    #[doc = r" Prompt for password."]
    pub ask_password: bool,
    #[doc = r" Environment option index."]
    pub environment: usize,
    #[doc = r" Group option index."]
    pub group: usize,
    #[doc = r" Safe-mode option index."]
    pub safe_mode: usize,
    #[doc = r" TLS toggle."]
    pub ssl: bool,
    #[doc = r" SSH tunnel toggle."]
    pub ssh: bool,
    #[doc = r" SSH host."]
    pub ssh_host: String,
    #[doc = r" Startup SQL."]
    pub startup: String,
}

impl ConnectionDraft {
    #[doc = r" Build a draft from an existing connection. Passwords are never copied."]
    pub fn from_connection(connection: &Connection) -> Self {
        Self {
            name: connection.name.clone(),
            engine: match connection.engine {
                Engine::Postgres => 0,
                Engine::MySql => 1,
                Engine::Sqlite => 2,
            },
            host: connection.host.clone(),
            port: connection.port.to_string(),
            database: connection.database.clone(),
            user: connection.user.clone(),
            password: Secret::default(),
            ask_password: false,
            environment: match connection.environment {
                Environment::Local => 0,
                Environment::Development => 1,
                Environment::Staging => 2,
                Environment::Production => 3,
            },
            group: GROUPS
                .iter()
                .position(|g| g.eq_ignore_ascii_case(&connection.group))
                .unwrap_or(0),
            safe_mode: SafeMode::ALL
                .iter()
                .position(|mode| *mode == connection.safe_mode)
                .unwrap_or(0),
            ssl: connection.ssl,
            ssh: connection.ssh.is_some(),
            ssh_host: connection.ssh.clone().unwrap_or_default(),
            startup: String::new(),
        }
    }

    #[doc = r" Validate required fields and the port."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns the first invalid field and its user-facing validation error."]
    pub fn validate_all(&self) -> Result<(), (Id, termrock::FieldError)> {
        use field::{DATABASE, NAME, PORT};
        if self.name.trim().is_empty() {
            return Err((
                NAME,
                termrock::FieldError::coded("Name is required", "required"),
            ));
        }
        if self.engine == 0 && self.database.trim().is_empty() {
            return Err((
                DATABASE,
                termrock::FieldError::coded("Database is required", "required"),
            ));
        }
        valid_port(&self.port).map_err(|e| (PORT, e))
    }

    #[doc = r" Build a connection value without persisting the password."]
    pub fn to_connection(&self, base: Option<&Connection>) -> Option<Connection> {
        let port = if self.port.is_empty() {
            default_port(self.engine).parse().ok()?
        } else {
            self.port.parse().ok()?
        };
        let engine = match self.engine {
            0 => Engine::Postgres,
            1 => Engine::MySql,
            2 => Engine::Sqlite,
            _ => return None,
        };
        let environment = match self.environment {
            0 => Environment::Local,
            1 => Environment::Development,
            2 => Environment::Staging,
            3 => Environment::Production,
            _ => return None,
        };
        Some(Connection {
            name: self.name.clone(),
            engine,
            host: self.host.clone(),
            port,
            database: self.database.clone(),
            user: self.user.clone(),
            environment,
            safe_mode: *SafeMode::ALL.get(self.safe_mode)?,
            ssl: self.ssl,
            ssh: self.ssh.then(|| self.ssh_host.clone()),
            group: GROUPS
                .get(self.group)
                .copied()
                .or_else(|| GROUPS.first().copied())
                .unwrap_or("Personal")
                .to_owned(),
            last_used: base.map_or_else(|| "never".to_owned(), |c| c.last_used.clone()),
            outcome: base.map_or(ConnectOutcome::Ok, |c| c.outcome),
        })
    }

    #[doc = r" Whether a password is present."]
    pub fn has_password(&self) -> bool {
        !self.password.is_empty()
    }
    #[doc = r" Redacted password status."]
    pub fn password_status(&self) -> &'static str {
        if self.has_password() {
            "saved"
        } else {
            "not set"
        }
    }

    fn options(id: Id) -> &'static [&'static str] {
        match id {
            field::ENGINE => ENGINES,
            field::ENVIRONMENT => ENVIRONMENTS,
            field::GROUP => GROUPS,
            field::SAFE_MODE => SAFE_MODES,
            _ => &[],
        }
    }
}

impl FormData for ConnectionDraft {
    fn options(&self, id: Id) -> &[&str] {
        Self::options(id)
    }

    fn value(&self, id: Id) -> FieldRef<'_> {
        use field::{
            ASK_PASSWORD, DATABASE, ENGINE, ENVIRONMENT, GROUP, HOST, NAME, PASSWORD, PORT,
            SAFE_MODE, SSH, SSH_HOST, SSL, STARTUP, USER,
        };
        match id {
            NAME => FieldRef::Text(&self.name),
            ENGINE => FieldRef::Choice(self.engine),
            HOST => FieldRef::Text(&self.host),
            PORT => FieldRef::Text(&self.port),
            DATABASE => FieldRef::Text(&self.database),
            USER => FieldRef::Text(&self.user),
            PASSWORD => FieldRef::Secret(&self.password),
            ASK_PASSWORD => FieldRef::Flag(self.ask_password),
            ENVIRONMENT => FieldRef::Choice(self.environment),
            GROUP => FieldRef::Choice(self.group),
            SAFE_MODE => FieldRef::Choice(self.safe_mode),
            SSL => FieldRef::Flag(self.ssl),
            SSH => FieldRef::Flag(self.ssh),
            SSH_HOST => FieldRef::Text(&self.ssh_host),
            STARTUP => FieldRef::Text(&self.startup),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        use field::{
            ASK_PASSWORD, DATABASE, ENGINE, ENVIRONMENT, GROUP, HOST, NAME, PASSWORD, PORT,
            SAFE_MODE, SSH, SSH_HOST, SSL, STARTUP, USER,
        };
        match id {
            NAME => FieldMut::Text(&mut self.name),
            ENGINE => FieldMut::Choice(&mut self.engine),
            HOST => FieldMut::Text(&mut self.host),
            PORT => FieldMut::Text(&mut self.port),
            DATABASE => FieldMut::Text(&mut self.database),
            USER => FieldMut::Text(&mut self.user),
            PASSWORD => FieldMut::Secret(&mut self.password),
            ASK_PASSWORD => FieldMut::Flag(&mut self.ask_password),
            ENVIRONMENT => FieldMut::Choice(&mut self.environment),
            GROUP => FieldMut::Choice(&mut self.group),
            SAFE_MODE => FieldMut::Choice(&mut self.safe_mode),
            SSL => FieldMut::Flag(&mut self.ssl),
            SSH => FieldMut::Flag(&mut self.ssh),
            SSH_HOST => FieldMut::Text(&mut self.ssh_host),
            STARTUP => FieldMut::Text(&mut self.startup),
            _ => FieldMut::ReadOnly,
        }
    }

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        let options = Self::options(id);
        (self.value_mut(id), options)
    }

    fn visible(&self, id: Id) -> bool {
        id != field::SSH_HOST || self.ssh
    }
    fn disabled(&self, id: Id) -> bool {
        id == field::PASSWORD && self.ask_password
    }
    fn validate(&self, id: Id, value: FieldRef<'_>) -> Result<(), termrock::FieldError> {
        if id == field::PORT
            && let FieldRef::Text(text) = value
        {
            valid_port(text)
        } else {
            Ok(())
        }
    }
    fn validate_all(&self) -> Result<(), (Id, termrock::FieldError)> {
        Self::validate_all(self)
    }
}

#[doc = r" Connection-list state."]
#[derive(Debug)]
pub struct ConnectionsScreen {
    #[doc = r" Configured rows."]
    pub connections: Vec<Connection>,
    #[doc = r" Selected visible-row index."]
    pub selected: usize,
    #[doc = r" Case-insensitive filter text."]
    pub filter: String,
    #[doc = r" Last connection error."]
    pub error: Option<String>,
}

impl ConnectionsScreen {
    #[doc = r" Build a connection list."]
    pub fn new(connections: Vec<Connection>) -> Self {
        Self {
            connections,
            selected: 0,
            filter: String::new(),
            error: None,
        }
    }
    #[doc = r" Build the demo list."]
    pub fn default_list() -> Self {
        Self::new(db::connections())
    }
    #[doc = r" Return rows visible under the current filter."]
    pub fn visible(&self) -> Vec<(usize, &Connection)> {
        let q = self.filter.to_ascii_lowercase();
        self.connections
            .iter()
            .enumerate()
            .filter(|(_, c)| q.is_empty() || c.name.to_ascii_lowercase().contains(&q))
            .collect()
    }
    #[doc = r" Connect the selected visible row."]
    pub fn connect_selected(&mut self) -> Option<Connection> {
        let index = self.visible().get(self.selected).map(|(i, _)| *i)?;
        let connection = self.connections.get(index)?.clone();
        match connection.outcome {
            ConnectOutcome::Ok => {
                self.error = None;
                Some(connection)
            }
            ConnectOutcome::AuthFailed => {
                self.error = Some("Authentication failed; press r to retry".to_owned());
                None
            }
            ConnectOutcome::Unreachable => {
                self.error = Some("Connection unreachable; press r to retry".to_owned());
                None
            }
        }
    }
    #[doc = r" Retry the selected row."]
    pub fn retry(&mut self) -> Option<Connection> {
        self.connect_selected()
    }
}
