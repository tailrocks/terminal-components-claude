//! Connection experience: grouped list, detail card, edit form with
//! BASIC / ADVANCED disclosure, simulated connect states.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::choice::{Checkbox, RadioGroup, Toggle};
use junie_tui::widgets::input::{InputEvent, TextInput};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::select::Select;
use junie_tui::widgets::tabs::Tabs;
use junie_tui::widgets::tree::{TreeEvent, TreeNode, TreeView};

use crate::db::{ConnectOutcome, Connection, Engine, Environment, SafeMode};

const ID: WidgetId = WidgetId::of("connections");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnState {
    Idle,
    Connecting {
        ticks: u32,
        name: String,
    },
    Failed {
        name: String,
        message: String,
        detail: String,
    },
    Testing {
        ticks: u32,
    },
    Tested(Result<String, String>),
}

pub enum ConnEvent {
    Connected(usize),
}

pub struct ConnectionsScreen {
    pub connections: Vec<Connection>,
    tree: TreeView,
    filter: TextInput,
    pub selected: Option<usize>,
    pub state: ConnState,
    connect_btn: Button,
    edit_btn: Button,
    dup_btn: Button,
    del_btn: Button,
    retry_btn: Button,
    form: Option<ConnForm>,
}

struct ConnForm {
    index: Option<usize>,
    tabs: Tabs,
    name: TextInput,
    engine: Select,
    host: TextInput,
    port: TextInput,
    database: TextInput,
    user: TextInput,
    password: TextInput,
    prompt_pw: Checkbox,
    env: RadioGroup,
    group: Select,
    safe: RadioGroup,
    // advanced
    ssl: Toggle,
    ssh: Toggle,
    ssh_host: TextInput,
    ssh_user: TextInput,
    startup: junie_tui::widgets::textarea::TextArea,
    local_only: Toggle,
    test_btn: Button,
    cancel_btn: Button,
    save_btn: Button,
    save_connect_btn: Button,
}

fn port_validator(s: &str) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    match s.parse::<u32>() {
        Ok(p) if p > 0 && p < 65536 => None,
        _ => Some("Port must be 1–65535".into()),
    }
}

fn name_validator(s: &str) -> Option<String> {
    if s.trim().is_empty() {
        Some("Required".into())
    } else {
        None
    }
}

impl ConnForm {
    fn new(c: Option<&Connection>, index: Option<usize>) -> Self {
        let f = ID.sub("form");
        let engines = ["PostgreSQL", "MySQL", "SQLite"];
        let eng = c
            .map(|c| match c.engine {
                Engine::Postgres => 0,
                Engine::MySql => 1,
                Engine::Sqlite => 2,
            })
            .unwrap_or(0);
        let env_i = c
            .map(|c| match c.environment {
                Environment::Local => 0,
                Environment::Development => 1,
                Environment::Staging => 2,
                Environment::Production => 3,
            })
            .unwrap_or(0);
        let safe_i = c
            .map(|c| {
                SafeMode::ALL
                    .iter()
                    .position(|s| *s == c.safe_mode)
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        let mut tabs = Tabs::new(f.sub("tabs"), &["Basic", "Advanced"]);
        tabs.active = 0;
        Self {
            index,
            tabs,
            name: TextInput::new(f.sub("name"), "Name")
                .required(true)
                .validator(name_validator)
                .value(c.map(|c| c.name.as_str()).unwrap_or("")),
            engine: Select::new(f.sub("engine"), "Engine", &engines, eng),
            host: TextInput::new(f.sub("host"), "Host")
                .value(c.map(|c| c.host.as_str()).unwrap_or("localhost"))
                .help("Blank: driver default"),
            port: TextInput::new(f.sub("port"), "Port")
                .value(&c.map(|c| c.port.to_string()).unwrap_or("5432".into()))
                .validator(port_validator),
            database: TextInput::new(f.sub("db"), "Database")
                .value(c.map(|c| c.database.as_str()).unwrap_or(""))
                .help("Required for PostgreSQL"),
            user: TextInput::new(f.sub("user"), "Username")
                .value(c.map(|c| c.user.as_str()).unwrap_or("")),
            password: TextInput::new(f.sub("pw"), "Password")
                .placeholder("stored in the keychain")
                .help("Never written to connections.json"),
            prompt_pw: Checkbox::new(f.sub("promptpw"), "Prompt for password on connect", false),
            env: RadioGroup::new(
                f.sub("env"),
                "Environment",
                &["local", "development", "staging", "production"],
                env_i,
            ),
            group: Select::new(
                f.sub("group"),
                "Group",
                &["Personal", "Acme", "Clients"],
                if c.is_some_and(|c| c.group == "Acme") {
                    1
                } else {
                    0
                },
            ),
            safe: RadioGroup::new(
                f.sub("safe"),
                "Safe Mode",
                &[
                    "Silent",
                    "Alert",
                    "Alert (Full)",
                    "Safe Mode",
                    "Safe Mode (Full)",
                    "Read-Only",
                ],
                safe_i,
            ),
            ssl: Toggle::new(f.sub("ssl"), "Use SSL / TLS", c.is_some_and(|c| c.ssl)),
            ssh: Toggle::new(
                f.sub("ssh"),
                "SSH tunnel",
                c.is_some_and(|c| c.ssh.is_some()),
            ),
            ssh_host: TextInput::new(f.sub("sshhost"), "SSH host")
                .value(c.and_then(|c| c.ssh.as_deref()).unwrap_or(""))
                .placeholder("bastion.example.com"),
            ssh_user: TextInput::new(f.sub("sshuser"), "SSH user").value("deploy"),
            startup: junie_tui::widgets::textarea::TextArea::new(
                f.sub("startup"),
                "Startup commands",
                3,
            )
            .placeholder("SET statement_timeout = '60s';")
            .help("Run after every connect, one per line"),
            local_only: Toggle::new(f.sub("local"), "Local only (no iCloud sync)", false),
            test_btn: Button::secondary(f.sub("test"), "Test connection"),
            cancel_btn: Button::subtle(f.sub("cancel"), "Cancel"),
            save_btn: Button::secondary(f.sub("save"), "Save"),
            save_connect_btn: Button::primary(f.sub("saveconnect"), "Save & connect"),
        }
    }

    fn to_connection(&self, base: Option<&Connection>) -> Connection {
        let engine = match self.engine.selected {
            0 => Engine::Postgres,
            1 => Engine::MySql,
            _ => Engine::Sqlite,
        };
        let environment = match self.env.selected {
            0 => Environment::Local,
            1 => Environment::Development,
            2 => Environment::Staging,
            _ => Environment::Production,
        };
        Connection {
            name: self.name.text().trim().to_owned(),
            engine,
            host: self.host.text().to_owned(),
            port: self.port.text().parse().unwrap_or(5432),
            database: self.database.text().to_owned(),
            user: self.user.text().to_owned(),
            environment,
            safe_mode: SafeMode::ALL[self.safe.selected.min(5)],
            ssl: self.ssl.on,
            ssh: if self.ssh.on && !self.ssh_host.text().is_empty() {
                Some(self.ssh_host.text().to_owned())
            } else {
                None
            },
            group: self.group.value().to_owned(),
            last_used: base.map(|b| b.last_used.clone()).unwrap_or("never".into()),
            outcome: base.map(|b| b.outcome).unwrap_or(ConnectOutcome::Ok),
        }
    }

    fn validate(&mut self) -> bool {
        let a = self.name.validate();
        let b = self.port.validate();
        a && b
    }
}

pub fn env_tone(env: Environment) -> Tone {
    match env {
        Environment::Production => Tone::Normal,
        Environment::Staging => Tone::Secondary,
        Environment::Development => Tone::Muted,
        Environment::Local => Tone::Faint,
    }
}

impl ConnectionsScreen {
    pub fn new(connections: Vec<Connection>) -> Self {
        let mut s = Self {
            connections,
            tree: TreeView::new(ID.sub("tree"), vec![]),
            filter: TextInput::new(ID.sub("filter"), "")
                .placeholder("Filter connections")
                .plain_label(),
            selected: Some(0),
            state: ConnState::Idle,
            connect_btn: Button::primary(ID.sub("connect"), "Connect"),
            edit_btn: Button::secondary(ID.sub("edit"), "Edit"),
            dup_btn: Button::subtle(ID.sub("dup"), "Duplicate"),
            del_btn: Button::danger(ID.sub("del"), "Delete…"),
            retry_btn: Button::secondary(ID.sub("retry"), "Retry"),
            form: None,
        };
        s.rebuild_tree();
        s
    }

    fn rebuild_tree(&mut self) {
        let mut groups: Vec<String> = Vec::new();
        for c in &self.connections {
            if !groups.contains(&c.group) {
                groups.push(c.group.clone());
            }
        }
        let nodes: Vec<TreeNode> = groups
            .iter()
            .map(|g| {
                let children: Vec<TreeNode> = self
                    .connections
                    .iter()
                    .filter(|c| &c.group == g)
                    .map(|c| {
                        let glyph = match c.environment {
                            Environment::Production => "◆",
                            Environment::Staging => "◇",
                            _ => "·",
                        };
                        TreeNode::leaf(&c.name).glyph(glyph).meta(c.engine.short())
                    })
                    .collect();
                TreeNode::dir(g, children)
            })
            .collect();
        let filter = self.tree.filter.clone();
        let cursor = self.tree.cursor;
        self.tree = TreeView::new(ID.sub("tree"), nodes);
        self.tree.expand_all();
        self.tree.filter = filter;
        self.tree.flatten();
        self.tree.cursor = cursor.min(self.tree.rows().len().saturating_sub(1));
        self.sync_selected_from_cursor();
    }

    fn connection_at_path(&self, path: &[usize]) -> Option<usize> {
        if path.len() != 2 {
            return None;
        }
        let node = self.tree.node(path)?;
        self.connections.iter().position(|c| c.name == node.label)
    }

    fn sync_selected_from_cursor(&mut self) {
        let rows = self.tree.rows();
        if let Some(row) = rows.get(self.tree.cursor)
            && let Some(i) = self.connection_at_path(&row.path.clone())
        {
            self.selected = Some(i);
        }
    }

    pub fn selected_connection(&self) -> Option<&Connection> {
        self.selected.and_then(|i| self.connections.get(i))
    }

    pub fn is_editing(&self) -> bool {
        if self.filter.editing {
            return true;
        }
        self.form.as_ref().is_some_and(|f| {
            f.name.editing
                || f.host.editing
                || f.port.editing
                || f.database.editing
                || f.user.editing
                || f.password.editing
                || f.ssh_host.editing
                || f.ssh_user.editing
                || f.startup.editing
        })
    }

    pub fn animating(&self) -> bool {
        matches!(
            self.state,
            ConnState::Connecting { .. } | ConnState::Testing { .. }
        )
    }

    pub fn start_connect(&mut self, i: usize) {
        if let Some(c) = self.connections.get(i) {
            self.selected = Some(i);
            self.state = ConnState::Connecting {
                ticks: 0,
                name: c.name.clone(),
            };
        }
    }

    pub fn open_form(&mut self, index: Option<usize>) {
        let c = index.and_then(|i| self.connections.get(i));
        self.form = Some(ConnForm::new(c, index));
    }

    pub fn open_new(&mut self) {
        self.open_form(None);
    }

    /// Advance simulated states. Returns a connected index when a connect completes.
    pub fn tick(&mut self) -> Option<ConnEvent> {
        match &mut self.state {
            ConnState::Connecting { ticks, name } => {
                *ticks += 1;
                if *ticks >= 12 {
                    let name = name.clone();
                    let idx = self.connections.iter().position(|c| c.name == name);
                    let outcome = idx
                        .map(|i| self.connections[i].outcome)
                        .unwrap_or(ConnectOutcome::Ok);
                    match outcome {
                        ConnectOutcome::Ok => {
                            self.state = ConnState::Idle;
                            if let Some(i) = idx {
                                self.connections[i].last_used = "just now".into();
                                return Some(ConnEvent::Connected(i));
                            }
                        }
                        ConnectOutcome::AuthFailed => {
                            self.state = ConnState::Failed {
                                name,
                                message: "Authentication failed".into(),
                                detail: "FATAL: password authentication failed for user \"acme_app\" (SQLSTATE 28P01). Check the password in the keychain or use “Prompt for password”.".into(),
                            };
                        }
                        ConnectOutcome::Unreachable => {
                            self.state = ConnState::Failed {
                                name,
                                message: "Could not reach the host".into(),
                                detail: "Connection timed out after 10 s (analytics.acme.io:3306). The host may be behind a VPN or the port may be blocked.".into(),
                            };
                        }
                    }
                }
            }
            ConnState::Testing { ticks } => {
                *ticks += 1;
                if *ticks >= 10 {
                    let ok = self
                        .form
                        .as_ref()
                        .is_some_and(|f| !f.host.text().contains("analytics"));
                    self.state = ConnState::Tested(if ok {
                        Ok("Connected · PostgreSQL 16.3 · 12 ms".into())
                    } else {
                        Err("Connection timed out after 10 s".into())
                    });
                }
            }
            _ => {}
        }
        None
    }

    // ---- input ---------------------------------------------------------

    pub fn on_key(&mut self, key: &Key, cx: &mut crate::app::Cx) -> (Outcome, Option<ConnEvent>) {
        let Some(f) = cx.focus.current() else {
            return (Outcome::Ignored, None);
        };
        if f == self.filter.id {
            let (o, ev) = self.filter.on_key(key);
            match ev {
                Some(InputEvent::Changed)
                | Some(InputEvent::Committed)
                | Some(InputEvent::Cancelled) => {
                    let q = self.filter.text().to_owned();
                    self.tree
                        .set_filter(if q.is_empty() { None } else { Some(&q) });
                    self.sync_selected_from_cursor();
                }
                Some(InputEvent::CommittedTab { backward }) => {
                    if backward {
                        cx.focus_prev()
                    } else {
                        cx.focus_next()
                    }
                }
                None => {}
            }
            if !o.consumed() && key.is(KeyCode::Down) {
                cx.focus.focus(self.tree.id);
                return (Outcome::Changed, None);
            }
            return (o, None);
        }
        if f == self.tree.id {
            if key.is_char('/') {
                cx.focus.focus(self.filter.id);
                self.filter.begin_edit();
                return (Outcome::Changed, None);
            }
            let (o, ev) = self.tree.on_key(key);
            self.sync_selected_from_cursor();
            if let Some(TreeEvent::Activate(path)) = ev
                && let Some(i) = self.connection_at_path(&path)
            {
                self.start_connect(i);
            }
            return (o, None);
        }
        if let Some(_form) = self.form.as_mut() {
            return (self.on_form_key(key, f, cx), None);
        }
        let mut hit: Option<(Outcome, bool, usize)> = None;
        for (b, action) in [
            (&mut self.connect_btn, 0),
            (&mut self.edit_btn, 1),
            (&mut self.dup_btn, 2),
            (&mut self.del_btn, 3),
            (&mut self.retry_btn, 4),
        ] {
            if b.id == f {
                let (o, act) = b.on_key(key);
                hit = Some((o, act, action));
                break;
            }
        }
        match hit {
            Some((_, true, action)) => (Outcome::Changed, self.action(action, cx)),
            Some((o, false, _)) => (o, None),
            None => (Outcome::Ignored, None),
        }
    }

    fn action(&mut self, which: usize, cx: &mut crate::app::Cx) -> Option<ConnEvent> {
        match which {
            0 | 4 => {
                if let Some(i) = self.selected {
                    self.start_connect(i);
                }
            }
            1 => {
                self.open_form(self.selected);
                if let Some(f) = &self.form {
                    cx.focus.focus(f.name.id);
                }
            }
            2 => {
                if let Some(i) = self.selected {
                    let mut c = self.connections[i].clone();
                    c.name = format!("{} (Copy)", c.name);
                    c.last_used = "never".into();
                    self.connections.insert(i + 1, c);
                    self.rebuild_tree();
                    cx.status("Duplicated");
                }
            }
            3 => {
                if let Some(c) = self.selected_connection() {
                    let d = junie_tui::widgets::dialog::Dialog::destructive(
                        ID.sub("delete-dialog"),
                        "Delete connection?",
                        &format!(
                            "{} ({}@{}) will be removed from connections.json. Its password stays in the keychain until you remove it there.",
                            c.name, c.user, c.host
                        ),
                        "Delete",
                    );
                    cx.open(d);
                }
            }
            _ => {}
        }
        None
    }

    pub fn on_dialog_closed(
        &mut self,
        id: WidgetId,
        result: junie_tui::widgets::dialog::DialogResult,
    ) -> bool {
        if id == ID.sub("delete-dialog") {
            if result == junie_tui::widgets::dialog::DialogResult::Action(1)
                && let Some(i) = self.selected
            {
                self.connections.remove(i);
                self.selected = if self.connections.is_empty() {
                    None
                } else {
                    Some(i.min(self.connections.len() - 1))
                };
                self.rebuild_tree();
            }
            return true;
        }
        false
    }

    fn on_form_key(&mut self, key: &Key, f: WidgetId, cx: &mut crate::app::Cx) -> Outcome {
        let Some(form) = self.form.as_mut() else {
            return Outcome::Ignored;
        };
        if key.ctrl_char('s') {
            return self.submit(false, cx);
        }
        if f == form.tabs.id {
            return form.tabs.on_key(key).0;
        }
        macro_rules! input {
            ($w:expr) => {{
                let (o, ev) = $w.on_key(key);
                match ev {
                    Some(InputEvent::CommittedTab { backward: false }) => cx.focus_next(),
                    Some(InputEvent::CommittedTab { backward: true }) => cx.focus_prev(),
                    _ => {}
                }
                return o;
            }};
        }
        if f == form.name.id {
            input!(form.name);
        }
        if f == form.host.id {
            input!(form.host);
        }
        if f == form.port.id {
            input!(form.port);
        }
        if f == form.database.id {
            input!(form.database);
        }
        if f == form.user.id {
            input!(form.user);
        }
        if f == form.password.id {
            input!(form.password);
        }
        if f == form.ssh_host.id {
            input!(form.ssh_host);
        }
        if f == form.ssh_user.id {
            input!(form.ssh_user);
        }
        if f == form.startup.id {
            input!(form.startup);
        }
        if f == form.engine.id {
            let (o, ev) = form.engine.on_key(key);
            if let Some(junie_tui::widgets::select::SelectEvent::Changed(i)) = ev {
                let port = match i {
                    0 => "5432",
                    1 => "3306",
                    _ => "",
                };
                form.port = TextInput::new(form.port.id, "Port")
                    .value(port)
                    .validator(port_validator);
            }
            return o;
        }
        if f == form.group.id {
            return form.group.on_key(key).0;
        }
        if f == form.env.id {
            return form.env.on_key(key);
        }
        if f == form.safe.id {
            return form.safe.on_key(key);
        }
        if f == form.prompt_pw.id {
            return form.prompt_pw.on_key(key);
        }
        if f == form.ssl.id {
            return form.ssl.on_key(key);
        }
        if f == form.ssh.id {
            return form.ssh.on_key(key);
        }
        if f == form.local_only.id {
            return form.local_only.on_key(key);
        }
        if f == form.test_btn.id {
            let (o, act) = form.test_btn.on_key(key);
            if act {
                self.state = ConnState::Testing { ticks: 0 };
            }
            return o;
        }
        if f == form.cancel_btn.id {
            let (o, act) = form.cancel_btn.on_key(key);
            if act {
                self.form = None;
                self.state = ConnState::Idle;
                cx.focus.focus(self.tree.id);
            }
            return o;
        }
        if f == form.save_btn.id {
            let (o, act) = form.save_btn.on_key(key);
            if act {
                return self.submit(false, cx);
            }
            return o;
        }
        if f == form.save_connect_btn.id {
            let (o, act) = form.save_connect_btn.on_key(key);
            if act {
                return self.submit(true, cx);
            }
            return o;
        }
        Outcome::Ignored
    }

    fn submit(&mut self, connect: bool, cx: &mut crate::app::Cx) -> Outcome {
        let Some(form) = self.form.as_mut() else {
            return Outcome::Ignored;
        };
        for inp in [
            &mut form.name,
            &mut form.host,
            &mut form.port,
            &mut form.database,
            &mut form.user,
        ] {
            if inp.editing {
                inp.commit();
            }
        }
        if !form.validate() {
            cx.status("Fix the highlighted fields");
            cx.focus.focus(if form.name.error.is_some() {
                form.name.id
            } else {
                form.port.id
            });
            return Outcome::Changed;
        }
        let base = form.index.and_then(|i| self.connections.get(i)).cloned();
        let c = form.to_connection(base.as_ref());
        let idx = match form.index {
            Some(i) => {
                self.connections[i] = c;
                i
            }
            None => {
                self.connections.push(c);
                self.connections.len() - 1
            }
        };
        self.form = None;
        self.state = ConnState::Idle;
        self.rebuild_tree();
        self.selected = Some(idx);
        cx.status("Connection saved");
        if connect {
            self.start_connect(idx);
        } else {
            cx.focus.focus(self.tree.id);
        }
        Outcome::Changed
    }

    pub fn on_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        cx: &mut crate::app::Cx,
    ) -> (Outcome, Option<ConnEvent>) {
        if id == self.filter.id {
            let was = cx.focus.is(id);
            cx.focus.focus(id);
            return (self.filter.on_click(pos, was), None);
        }
        if let Some((row, toggle)) = self.tree.locate(id) {
            cx.focus.focus(self.tree.id);
            let (o, ev) = if toggle {
                self.tree.on_click_toggle(row)
            } else {
                self.tree.on_click_row(row)
            };
            self.sync_selected_from_cursor();
            if let Some(TreeEvent::Activate(path)) = ev {
                // single click selects; Enter/second click connects
                if let Some(i) = self.connection_at_path(&path)
                    && self.selected == Some(i)
                    && self.tree.selected.as_ref() == Some(&path)
                    && o == Outcome::Changed
                    && self.state == ConnState::Idle
                {
                    // second click on an already-selected row: connect
                }
            }
            return (o, None);
        }
        if self.form.is_some() {
            return (self.on_form_click(id, pos, cx), None);
        }
        let mut hit: Option<(bool, usize)> = None;
        for (b, action) in [
            (&mut self.connect_btn, 0),
            (&mut self.edit_btn, 1),
            (&mut self.dup_btn, 2),
            (&mut self.del_btn, 3),
            (&mut self.retry_btn, 4),
        ] {
            if b.id == id {
                hit = Some((b.on_click(), action));
                break;
            }
        }
        match hit {
            Some((true, action)) => (Outcome::Changed, self.action(action, cx)),
            Some((false, _)) => (Outcome::Changed, None),
            None => (Outcome::Ignored, None),
        }
    }

    fn on_form_click(&mut self, id: WidgetId, pos: Position, cx: &mut crate::app::Cx) -> Outcome {
        let Some(form) = self.form.as_mut() else {
            return Outcome::Ignored;
        };
        if form.tabs.locate(id).is_some() {
            cx.focus.focus(form.tabs.id);
            return form.tabs.on_click(id).0;
        }
        for inp in [
            &mut form.name,
            &mut form.host,
            &mut form.port,
            &mut form.database,
            &mut form.user,
            &mut form.password,
            &mut form.ssh_host,
            &mut form.ssh_user,
        ] {
            if inp.id == id {
                let was = cx.focus.is(id);
                cx.focus.focus(id);
                return inp.on_click(pos, was);
            }
        }
        if form.startup.id == id {
            let was = cx.focus.is(id);
            cx.focus.focus(id);
            return form.startup.on_click(pos, was);
        }
        for sel in [&mut form.engine, &mut form.group] {
            if sel.owns(id) {
                cx.focus.focus(sel.id);
                return sel.on_click(id).0;
            }
        }
        for rg in [&mut form.env, &mut form.safe] {
            for i in 0..rg.options.len() {
                if rg.option_id(i) == id {
                    cx.focus.focus(rg.id);
                    return rg.on_click(i);
                }
            }
        }
        if form.prompt_pw.id == id {
            return form.prompt_pw.on_click();
        }
        for tg in [&mut form.ssl, &mut form.ssh, &mut form.local_only] {
            if tg.id == id {
                return tg.on_click();
            }
        }
        if form.test_btn.id == id && form.test_btn.on_click() {
            self.state = ConnState::Testing { ticks: 0 };
            return Outcome::Changed;
        }
        if form.cancel_btn.id == id && form.cancel_btn.on_click() {
            self.form = None;
            self.state = ConnState::Idle;
            cx.focus.focus(self.tree.id);
            return Outcome::Changed;
        }
        if form.save_btn.id == id && form.save_btn.on_click() {
            return self.submit(false, cx);
        }
        if form.save_connect_btn.id == id && form.save_connect_btn.on_click() {
            return self.submit(true, cx);
        }
        Outcome::Ignored
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(form) = self.form.as_mut() {
            for inp in [
                &mut form.name,
                &mut form.host,
                &mut form.port,
                &mut form.database,
                &mut form.user,
                &mut form.password,
                &mut form.ssh_host,
                &mut form.ssh_user,
            ] {
                if inp.editing {
                    return inp.on_paste(text);
                }
            }
            if form.startup.editing {
                return form.startup.on_paste(text);
            }
        }
        if self.filter.editing {
            return self.filter.on_paste(text);
        }
        Outcome::Ignored
    }

    pub fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if self.form.is_some() {
            if self.is_editing() {
                return vec![
                    hint("Enter", "Commit"),
                    hint("Esc", "Cancel"),
                    hint("Tab", "Next field"),
                ];
            }
            return vec![
                hint("Enter", "Edit"),
                hint("← →", "Basic / Advanced"),
                hint("Ctrl+S", "Save"),
            ];
        }
        if focus == Some(self.filter.id) {
            return vec![
                hint("Type", "Filter"),
                hint("↓", "Into list"),
                hint("Esc", "Clear"),
            ];
        }
        vec![
            hint("↑ ↓", "Move"),
            hint("Enter", "Connect"),
            hint("/", "Filter"),
            hint("Ctrl+N", "New"),
        ]
    }

    // ---- render --------------------------------------------------------

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let list_w = (area.width / 3).clamp(26, 40);
        let (l, r) = if area.width >= 80 {
            (
                Rect::new(area.x, area.y, list_w, area.height),
                Rect::new(
                    area.x + list_w + 2,
                    area.y,
                    area.width.saturating_sub(list_w + 2),
                    area.height,
                ),
            )
        } else {
            (area, Rect::ZERO)
        };
        // list pane
        let lf = ctx.interaction.focused(self.tree.id) || ctx.interaction.focused(self.filter.id);
        let count = format!("{}", self.connections.len());
        let panel = Panel::framed(Some("Connections")).focused(lf).meta(&count);
        let bg = panel.bg(t);
        let inner = panel.render(l, buf, t);
        self.filter.render(
            Rect::new(inner.x.saturating_sub(1), inner.y, inner.width + 1, 2),
            buf,
            ctx,
            bg,
        );
        let tree_area = Rect::new(
            inner.x.saturating_sub(1),
            inner.y + 2,
            inner.width + 1,
            inner.height.saturating_sub(2),
        );
        if self.connections.is_empty() {
            junie_tui::widgets::empty::render(
                tree_area,
                buf,
                t,
                &junie_tui::widgets::empty::EmptyState::new("No connections")
                    .hint("Ctrl+N creates one"),
                bg,
            );
        } else {
            self.tree.render(tree_area, buf, ctx, bg);
        }
        if r.is_empty() {
            return;
        }
        if self.form.is_some() {
            self.render_form(r, buf, ctx);
            return;
        }
        // detail card
        let Some(c) = self.selected_connection().cloned() else {
            junie_tui::widgets::empty::render(
                r,
                buf,
                t,
                &junie_tui::widgets::empty::EmptyState::new("Select a connection"),
                t.canvas,
            );
            return;
        };
        let card_h = 17.min(r.height);
        let panel = Panel::card(Some(&c.name));
        let bg = panel.bg(t);
        let inner = panel.render(Rect::new(r.x, r.y, r.width.min(70), card_h), buf, t);
        let ssl = if c.ssl { "on" } else { "off" };
        let ssh = c.ssh.clone().unwrap_or("off".into());
        let mut facts = vec![
            Prop::new("Engine", c.engine.label()),
            Prop::new(
                "Host",
                if c.port > 0 {
                    format!("{}:{}", c.host, c.port)
                } else {
                    c.host.clone()
                },
            ),
            Prop::new(
                "Database",
                if c.database.is_empty() {
                    "—".into()
                } else {
                    c.database.clone()
                },
            ),
            Prop::new(
                "User",
                if c.user.is_empty() {
                    "—".into()
                } else {
                    c.user.clone()
                },
            ),
            Prop::new("Environment", c.environment.label()).tone(env_tone(c.environment)),
            Prop::new(
                "Safe Mode",
                format!("{} · {}", c.safe_mode.label(), c.safe_mode.description()),
            )
            .tone(if c.safe_mode >= SafeMode::Safe {
                Tone::Normal
            } else {
                Tone::Secondary
            })
            .wrap(),
            Prop::new("SSL / SSH", format!("{ssl} / {ssh}")).tone(Tone::Secondary),
            Prop::new("Last used", c.last_used.clone()).tone(Tone::Muted),
        ];
        if c.environment == Environment::Production && c.safe_mode == SafeMode::Silent {
            facts.insert(
                6,
                Prop::new(
                    "",
                    "Production with Silent safe mode: writes run without asking",
                )
                .tone(Tone::Warning)
                .wrap(),
            );
        }
        let used = props::render(
            Rect::new(
                inner.x,
                inner.y,
                inner.width,
                inner.height.saturating_sub(3),
            ),
            buf,
            t,
            &facts,
            bg,
        );
        // state line
        let sy = inner.y + used + 1;
        match &self.state {
            ConnState::Connecting { ticks, name } if *name == c.name => {
                let phase = match ticks {
                    0..=3 => "Opening SSH tunnel…",
                    4..=7 => "Authenticating…",
                    _ => "Loading schema…",
                };
                junie_tui::widgets::progress::render_spinner(
                    Rect::new(inner.x, sy, inner.width, 1),
                    buf,
                    ctx,
                    phase,
                    bg,
                );
            }
            ConnState::Failed {
                name,
                message,
                detail,
            } if *name == c.name => {
                buf.set_string(
                    inner.x,
                    sy,
                    "!",
                    t.error_fg()
                        .bg(bg)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                );
                buf.set_string(inner.x + 2, sy, message, t.error_fg().bg(bg));
                for (i, l) in
                    junie_tui::ui::text::wrap(detail, inner.width.saturating_sub(2) as usize)
                        .iter()
                        .take(2)
                        .enumerate()
                {
                    buf.set_string(inner.x + 2, sy + 1 + i as u16, l, t.muted().bg(bg));
                }
            }
            _ => {}
        }
        // actions
        let ay = inner.bottom().saturating_sub(1);
        let connecting =
            matches!(&self.state, ConnState::Connecting { name, .. } if *name == c.name);
        let failed = matches!(&self.state, ConnState::Failed { name, .. } if *name == c.name);
        self.connect_btn.busy = connecting;
        self.connect_btn.label = if failed {
            "Reconnect".into()
        } else {
            "Connect".into()
        };
        let widths = [
            self.connect_btn.width(),
            self.edit_btn.width(),
            self.dup_btn.width(),
            self.del_btn.width(),
        ];
        let rects = row_layout(Rect::new(inner.x, ay, inner.width, 1), &widths, 2);
        self.connect_btn.render(rects[0], buf, ctx, bg);
        self.edit_btn.render(rects[1], buf, ctx, bg);
        self.dup_btn.render(rects[2], buf, ctx, bg);
        self.del_btn.render(rects[3], buf, ctx, bg);
    }

    fn render_form(&mut self, r: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let Some(form) = self.form.as_mut() else {
            return;
        };
        let title = if form.index.is_some() {
            "Edit connection"
        } else {
            "New connection"
        };
        let panel = Panel::card(Some(title)).meta("Ctrl+S Save");
        let bg = panel.bg(t);
        let inner = panel.render(Rect::new(r.x, r.y, r.width.min(84), r.height), buf, t);
        // mark tabs with required-field errors
        form.tabs.items[0].error = form.name.error.is_some() || form.port.error.is_some();
        form.tabs
            .render(Rect::new(inner.x, inner.y, inner.width, 2), buf, ctx, bg);
        let body = Rect::new(
            inner.x,
            inner.y + 3,
            inner.width,
            inner.height.saturating_sub(5),
        );
        let (lc, rc) = junie_tui::ui::layout::Split::new(50, 24, 24).horizontal(body, 4);
        let fh = TextInput::HEIGHT;
        if form.tabs.active == 0 {
            let mut y = lc.y;
            form.name
                .render(Rect::new(lc.x, y, lc.width, fh), buf, ctx, bg);
            y += fh;
            form.engine
                .render(Rect::new(lc.x, y, lc.width, Select::HEIGHT), buf, ctx, bg);
            y += Select::HEIGHT;
            let (hl, hr) = junie_tui::ui::layout::Split::new(70, 12, 8)
                .horizontal(Rect::new(lc.x, y, lc.width, fh), 2);
            form.host.render(hl, buf, ctx, bg);
            form.port.render(hr, buf, ctx, bg);
            y += fh;
            form.database
                .render(Rect::new(lc.x, y, lc.width, fh), buf, ctx, bg);
            y += fh;
            form.user
                .render(Rect::new(lc.x, y, lc.width, fh), buf, ctx, bg);
            y += fh;
            form.password
                .render(Rect::new(lc.x, y, lc.width, fh), buf, ctx, bg);
            y += fh;
            form.prompt_pw
                .render(Rect::new(lc.x, y, lc.width, 1), buf, ctx, bg);
            let mut y = rc.y;
            form.env.render(
                Rect::new(rc.x, y, rc.width, form.env.height()),
                buf,
                ctx,
                bg,
            );
            y += form.env.height() + 1;
            form.group
                .render(Rect::new(rc.x, y, rc.width, Select::HEIGHT), buf, ctx, bg);
            y += Select::HEIGHT;
            form.safe.render(
                Rect::new(rc.x, y, rc.width, form.safe.height()),
                buf,
                ctx,
                bg,
            );
            y += form.safe.height();
            let desc = SafeMode::ALL[form.safe.selected.min(5)].description();
            for (i, l) in junie_tui::ui::text::wrap(desc, rc.width.saturating_sub(2) as usize)
                .iter()
                .take(2)
                .enumerate()
            {
                if y + (i as u16) < body.bottom() {
                    buf.set_string(rc.x + 2, y + i as u16, l, t.muted().bg(bg));
                }
            }
        } else {
            let mut y = lc.y;
            form.ssl
                .render(Rect::new(lc.x, y, lc.width, 1), buf, ctx, bg);
            y += 2;
            form.ssh
                .render(Rect::new(lc.x, y, lc.width, 1), buf, ctx, bg);
            y += 1;
            form.ssh_host.disabled = !form.ssh.on;
            form.ssh_user.disabled = !form.ssh.on;
            form.ssh_host
                .render(Rect::new(lc.x, y, lc.width, fh), buf, ctx, bg);
            y += fh;
            form.ssh_user
                .render(Rect::new(lc.x, y, lc.width, fh), buf, ctx, bg);
            y += fh;
            form.local_only
                .render(Rect::new(lc.x, y, lc.width, 1), buf, ctx, bg);
            form.startup.render(
                Rect::new(rc.x, rc.y, rc.width, form.startup.height()),
                buf,
                ctx,
                bg,
            );
            let y2 = rc.y + form.startup.height() + 1;
            buf.set_string(
                rc.x + 2,
                y2,
                "External clients: read only",
                t.muted().bg(bg),
            );
        }
        // footer: test + actions
        let ay = inner.bottom().saturating_sub(1);
        form.test_btn.busy = matches!(self.state, ConnState::Testing { .. });
        let widths = [
            form.test_btn.width(),
            form.cancel_btn.width(),
            form.save_btn.width(),
            form.save_connect_btn.width(),
        ];
        let rects = row_layout(Rect::new(inner.x, ay, inner.width, 1), &widths, 2);
        form.test_btn.render(rects[0], buf, ctx, bg);
        form.cancel_btn.render(rects[1], buf, ctx, bg);
        form.save_btn.render(rects[2], buf, ctx, bg);
        form.save_connect_btn.render(rects[3], buf, ctx, bg);
        if let ConnState::Tested(res) = &self.state {
            let (msg, st) = match res {
                Ok(m) => (format!("✓ {m}"), t.secondary()),
                Err(e) => (format!("! {e}"), t.error_fg()),
            };
            let x = rects[3].right() + 2;
            buf.set_string(
                x,
                ay,
                junie_tui::ui::text::truncate(&msg, inner.right().saturating_sub(x) as usize),
                st.bg(bg),
            );
        }
    }
}
