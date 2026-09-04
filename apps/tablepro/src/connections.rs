//! Connection list and the app-owned 15-field connection draft.

use tui_next::{
    Action, ActionKey, Checkbox, Chord, FieldKind, FieldMut, FieldRef, FieldSpan, FieldSpec,
    FormData, GroupKey, Id, RadioGroup, Secret, SecretPolicy, Select, TextArea, TextInput, Toggle,
};

use crate::db::{self, ConnectOutcome, Connection, Engine, Environment, SafeMode};

/// Connection-list id.
pub const CONNECTIONS: Id = Id::root("tablepro.connections");
/// Connection form id.
pub const FORM: Id = Id::root("tablepro.connections.form");

/// Stable form field ids.
pub mod field {
    use tui_next::Id;
    pub const NAME: Id = Id::root("tablepro.connections.form.name");
    pub const ENGINE: Id = Id::root("tablepro.connections.form.engine");
    pub const HOST: Id = Id::root("tablepro.connections.form.host");
    pub const PORT: Id = Id::root("tablepro.connections.form.port");
    pub const DATABASE: Id = Id::root("tablepro.connections.form.database");
    pub const USER: Id = Id::root("tablepro.connections.form.user");
    pub const PASSWORD: Id = Id::root("tablepro.connections.form.password");
    pub const ASK_PASSWORD: Id = Id::root("tablepro.connections.form.ask-password");
    pub const ENVIRONMENT: Id = Id::root("tablepro.connections.form.environment");
    pub const GROUP: Id = Id::root("tablepro.connections.form.group");
    pub const SAFE_MODE: Id = Id::root("tablepro.connections.form.safe-mode");
    pub const SSL: Id = Id::root("tablepro.connections.form.ssl");
    pub const SSH: Id = Id::root("tablepro.connections.form.ssh");
    pub const SSH_HOST: Id = Id::root("tablepro.connections.form.ssh-host");
    pub const STARTUP: Id = Id::root("tablepro.connections.form.startup");
}

/// Basic connection fields group.
pub const BASIC: GroupKey = GroupKey::custom("tablepro.connections.basic");
/// Advanced connection fields group.
pub const ADVANCED: GroupKey = GroupKey::custom("tablepro.connections.advanced");
/// Test action.
pub const TEST: ActionKey = ActionKey::custom("tablepro.connections.test");
/// Save-and-connect action.
pub const SAVE_CONNECT: ActionKey = ActionKey::custom("tablepro.connections.save-connect");

/// Engine option labels.
pub const ENGINES: &[&str] = &["PostgreSQL", "MySQL", "SQLite"];
/// Environment option labels.
pub const ENVIRONMENTS: &[&str] = &["Local", "Development", "Staging", "Production"];
/// Group option labels.
pub const GROUPS: &[&str] = &["Personal", "Acme"];
/// Safe-mode option labels.
pub const SAFE_MODES: &[&str] = &[
    "Silent",
    "Alert",
    "Alert (Full)",
    "Safe Mode",
    "Safe Mode (Full)",
    "Read-Only",
];

/// Default port for an engine choice.
pub const fn default_port(engine: usize) -> &'static str {
    match engine {
        0 => "5432",
        1 => "3306",
        _ => "",
    }
}

fn valid_port(value: &str) -> Result<(), tui_next::FieldError> {
    if value.is_empty() || value.parse::<u16>().is_ok() {
        Ok(())
    } else {
        Err(tui_next::FieldError::coded("Enter a valid port", "port"))
    }
}

/// The public `Form` declaration for the complete connection editor.
pub fn form_fields() -> [FieldSpec<'static>; 15] {
    use field::*;
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
        .span(FieldSpan::Half)
        .group(BASIC),
        FieldSpec::new(
            PORT,
            "Port",
            FieldKind::Text(TextInput::new(PORT).validate(&valid_port)),
        )
        .span(FieldSpan::Half)
        .group(BASIC),
        FieldSpec::new(
            DATABASE,
            "Database",
            FieldKind::Text(TextInput::new(DATABASE)),
        )
        .group(BASIC),
        FieldSpec::new(USER, "Username", FieldKind::Text(TextInput::new(USER))).group(BASIC),
        FieldSpec::new(
            PASSWORD,
            "Password",
            FieldKind::Text(TextInput::new(PASSWORD).secret(SecretPolicy::default())),
        )
        .help("Never written to connections.json")
        .group(BASIC),
        FieldSpec::new(
            ASK_PASSWORD,
            "",
            FieldKind::Check(Checkbox::new(
                ASK_PASSWORD,
                "Prompt for password on connect",
            )),
        )
        .plain(true)
        .group(BASIC),
        FieldSpec::new(
            ENVIRONMENT,
            "Environment",
            FieldKind::Radio(RadioGroup::new(ENVIRONMENT)),
        )
        .group(BASIC),
        FieldSpec::new(GROUP, "Group", FieldKind::Select(Select::new(GROUP))).group(BASIC),
        FieldSpec::new(
            SAFE_MODE,
            "Safe Mode",
            FieldKind::Radio(RadioGroup::new(SAFE_MODE)),
        )
        .group(BASIC),
        FieldSpec::new(
            SSL,
            "",
            FieldKind::Toggle(Toggle::new(SSL, "Use SSL / TLS")),
        )
        .plain(true)
        .group(ADVANCED),
        FieldSpec::new(SSH, "", FieldKind::Toggle(Toggle::new(SSH, "SSH tunnel")))
            .plain(true)
            .group(ADVANCED),
        FieldSpec::new(
            SSH_HOST,
            "SSH host",
            FieldKind::Text(TextInput::new(SSH_HOST).placeholder("bastion.example.com")),
        )
        .group(ADVANCED),
        FieldSpec::new(
            STARTUP,
            "Startup commands",
            FieldKind::Area(TextArea::new(STARTUP, 3)),
        )
        .help("Run after every connect, one per line")
        .group(ADVANCED),
    ]
}

/// The public `Form` action row.
pub fn form_actions() -> [Action<'static>; 4] {
    [
        Action::quiet(TEST, "Test connection"),
        Action::new(ActionKey::CANCEL, "Cancel"),
        Action::new(ActionKey::SAVE, "Save").chord(Chord::with(
            tui_next::KeyCode::Char('s'),
            tui_next::KeyModifiers::CONTROL,
        )),
        Action::new(SAVE_CONNECT, "Save & connect"),
    ]
}

/// Controlled connection form data. Password storage is a `Secret`, so this
/// type intentionally has no `Clone`, `Eq`, or string-producing debug path.
#[derive(Debug, Default)]
pub struct ConnectionDraft {
    /// Display name.
    pub name: String,
    /// Engine option index.
    pub engine: usize,
    /// Host name.
    pub host: String,
    /// Port text.
    pub port: String,
    /// Database name.
    pub database: String,
    /// User name.
    pub user: String,
    /// Password.
    pub password: Secret,
    /// Prompt for password.
    pub ask_password: bool,
    /// Environment option index.
    pub environment: usize,
    /// Group option index.
    pub group: usize,
    /// Safe-mode option index.
    pub safe_mode: usize,
    /// TLS toggle.
    pub ssl: bool,
    /// SSH tunnel toggle.
    pub ssh: bool,
    /// SSH host.
    pub ssh_host: String,
    /// Startup SQL.
    pub startup: String,
}

impl ConnectionDraft {
    /// Build a draft from an existing connection. Passwords are never copied.
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

    /// Validate required fields and the port.
    pub fn validate_all(&self) -> Result<(), (Id, tui_next::FieldError)> {
        use field::*;
        if self.name.trim().is_empty() {
            return Err((
                NAME,
                tui_next::FieldError::coded("Name is required", "required"),
            ));
        }
        if self.engine == 0 && self.database.trim().is_empty() {
            return Err((
                DATABASE,
                tui_next::FieldError::coded("Database is required", "required"),
            ));
        }
        valid_port(&self.port).map_err(|e| (PORT, e))
    }

    /// Build a connection value without persisting the password.
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

    /// Whether a password is present.
    pub fn has_password(&self) -> bool {
        !self.password.is_empty()
    }
    /// Redacted password status.
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
    fn value(&self, id: Id) -> FieldRef<'_> {
        use field::*;
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
        use field::*;
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
    fn validate(&self, id: Id, value: FieldRef<'_>) -> Result<(), tui_next::FieldError> {
        if id == field::PORT
            && let FieldRef::Text(text) = value
        {
            valid_port(text)
        } else {
            Ok(())
        }
    }
    fn validate_all(&self) -> Result<(), (Id, tui_next::FieldError)> {
        Self::validate_all(self)
    }
}

/// Connection-list state.
#[derive(Debug)]
pub struct ConnectionsScreen {
    /// Configured rows.
    pub connections: Vec<Connection>,
    /// Selected visible-row index.
    pub selected: usize,
    /// Case-insensitive filter text.
    pub filter: String,
    /// Last connection error.
    pub error: Option<String>,
}

impl ConnectionsScreen {
    /// Build a connection list.
    pub fn new(connections: Vec<Connection>) -> Self {
        Self {
            connections,
            selected: 0,
            filter: String::new(),
            error: None,
        }
    }
    /// Build the demo list.
    pub fn default_list() -> Self {
        Self::new(db::connections())
    }
    /// Return rows visible under the current filter.
    pub fn visible(&self) -> Vec<(usize, &Connection)> {
        let q = self.filter.to_ascii_lowercase();
        self.connections
            .iter()
            .enumerate()
            .filter(|(_, c)| q.is_empty() || c.name.to_ascii_lowercase().contains(&q))
            .collect()
    }
    /// Connect the selected visible row.
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
    /// Retry the selected row.
    pub fn retry(&mut self) -> Option<Connection> {
        self.connect_selected()
    }
}
