//! Product dialog intents projected through the shared Dialog and Props controls.
use super::context::Context;
use crate::dispatch::{self, Destination};
use crate::domain::{
    action::{Action, Availability},
    github::GhRepo,
    ssh::SshHost,
};
use crate::sim::{catalogue, pg, world::World};
use junie_tui::{
    Action as UiAction, ActionKey, Cx, Dialog, DialogAction, DialogState, Id, Props, Rect,
    Response, TextViewport, Ui, ViewportLine, ViewportState,
};

pub(crate) const DIALOG: Id = Id::root("holla.dialog");
const CODE: Id = Id::root("holla.dialog.code");
const TERMINATE: ActionKey = ActionKey::application("holla.pg.terminate");

pub(crate) enum Intent {
    About,
    Help,
    Quit,
    Monitor,
    Preview(Action),
    Trust {
        action: Action,
        file: String,
    },
    Database {
        cancel: Option<pg::Review>,
        terminate: Option<pg::Review>,
    },
    Clone {
        repo: GhRepo,
        login: String,
    },
    Alias(Action),
}
pub(crate) enum Event {
    Close,
    Quit,
    Destination(Destination),
}
pub(crate) struct ProductDialog {
    intent: Intent,
    context: Context,
    ssh: Option<SshHost>,
    width: u16,
    title: String,
    description: String,
    facts: Vec<(String, String)>,
    commands: Vec<String>,
    actions: Vec<UiAction<'static>>,
    dialog: DialogState,
    output: ViewportState,
}
impl ProductDialog {
    pub(crate) fn new(intent: Intent, world: &World) -> Self {
        let mut view = Self {
            intent,
            context: Context::of(world),
            ssh: None,
            width: 88,
            title: String::new(),
            description: String::new(),
            facts: Vec::new(),
            commands: Vec::new(),
            actions: vec![UiAction::quiet(ActionKey::CANCEL, "Close")],
            dialog: DialogState::default(),
            output: ViewportState::default(),
        };
        match &view.intent {
            Intent::About => {
                view.title = "About holla".into();
                view.facts = facts(&[
                    ("Product", "holla · context-adaptive action launcher".into()),
                    ("Scenario", world.scenario.name().into()),
                    (
                        "Stack",
                        "simulated · mise, git, docker, ssh are never executed".into(),
                    ),
                    (
                        "Design system",
                        "Junie-inspired Ratatui components (junie_tui)".into(),
                    ),
                ]);
            }
            Intent::Help => {
                view.title = "Key reference".into();
                view.facts = facts(&[
                    ("Type", "filter the action list".into()),
                    ("↑ ↓", "move between query and results".into()),
                    ("Enter", "run the focused action".into()),
                    ("Ctrl+O / Ctrl+P", "actions menu / preview".into()),
                    ("Esc", "clear query · close dialog".into()),
                    ("F10", "menu bar".into()),
                    ("?", "this reference (empty query)".into()),
                    ("q", "quit (empty query)".into()),
                ]);
            }
            Intent::Quit => {
                view.title = if world.host.env.sensitive() {
                    format!("Quit holla on {}?", world.host.name)
                } else {
                    "Quit holla?".into()
                };
                view.description = if world.host.env.sensitive() {
                    format!(
                        "You are on {}. The simulated session ends; nothing on that host changes.",
                        world.host.identity()
                    )
                } else {
                    "The simulated session ends; nothing on this host changes.".into()
                };
                view.actions = vec![
                    UiAction::quiet(ActionKey::CANCEL, "Cancel"),
                    UiAction::new(ActionKey::CONFIRM, "Quit"),
                ];
            }
            Intent::Preview(action) => {
                view.title.clone_from(&action.title);
                let workdir = if action.workdir.is_empty() {
                    &action.target
                } else {
                    &action.workdir
                };
                view.facts = facts(&[
                    ("Will happen", format!("{} · in {workdir}", action.command)),
                    (
                        "Target",
                        format!("{} · {}", action.scope.label(), action.target),
                    ),
                    ("Why recommended", action.reason.clone()),
                    ("Will change", action.changes()),
                    (
                        "Freshness",
                        "discovered at launch · simulated fixture, deterministic".into(),
                    ),
                    ("Confirmation", action.risk.confirmation().into()),
                ]);
                view.commands.push(action.command.clone());
                view.actions.push(
                    UiAction::new(ActionKey::CONFIRM, "Run (simulated)")
                        .enabled(matches!(action.availability, Availability::Ready)),
                );
            }
            Intent::Trust { action, file } => {
                view.title = "Trust this task file?".into();
                view.facts = facts(&[
                    ("Config file", file.clone()),
                    (
                        "Defines",
                        format!(
                            "{} → {}",
                            action
                                .intent_id()
                                .unwrap_or(&action.id)
                                .trim_start_matches("task:"),
                            action.command
                        ),
                    ),
                    ("Effect", "its tasks become runnable on this host".into()),
                    ("Trust scope", "this exact file only · revocable".into()),
                    ("Why asked", "mise refuses untrusted task files".into()),
                ]);
                view.commands.push(action.command.clone());
                view.actions
                    .push(UiAction::new(ActionKey::CONFIRM, "Trust file"));
            }
            Intent::Monitor => {
                view.title = "System snapshot".into();
                let metrics = &world.host.metrics;
                let (a, b, c) = metrics.load_x100;
                let disk = world.disk.as_ref().map_or_else(
                    || "not scanned".into(),
                    |disk| {
                        format!(
                            "{}% used · {} of {}",
                            disk.used_percent(),
                            crate::domain::human_bytes(disk.used_bytes),
                            crate::domain::human_bytes(disk.total_bytes)
                        )
                    },
                );
                let docker = world.docker.as_ref().map_or_else(
                    || "not discovered".into(),
                    |docker| {
                        format!(
                            "{} containers · {} running",
                            docker.containers.len(),
                            docker.running()
                        )
                    },
                );
                view.facts = facts(&[
                    ("Host", world.host.identity()),
                    (
                        "Load",
                        format!(
                            "{}.{:02} {}.{:02} {}.{:02}",
                            a / 100,
                            a % 100,
                            b / 100,
                            b % 100,
                            c / 100,
                            c % 100
                        ),
                    ),
                    (
                        "Memory",
                        format!("{} MB of {} MB", metrics.mem_used_mb, metrics.mem_total_mb),
                    ),
                    ("Disk /", disk),
                    ("Docker", docker),
                    ("Uptime", format!("{} days", metrics.uptime_days)),
                    (
                        "Handoff",
                        "btm takes over the screen for live monitoring · holla returns on exit"
                            .into(),
                    ),
                ]);
                view.commands.push("btm".into());
                view.actions
                    .push(UiAction::new(ActionKey::CONFIRM, "Open btm (simulated)"));
            }
            Intent::Database { cancel, .. } => {
                view.title = "Database lock tree".into();
                if let Some(review) = cancel {
                    let root = review.target();
                    let waiting = world.pg.as_ref().map_or_else(String::new, |sessions| {
                        crate::domain::pg::blocked_by(sessions, root.pid)
                            .iter()
                            .map(|session| {
                                format!(
                                    "pid {} {} · {}m",
                                    session.pid,
                                    session.user,
                                    session.duration_ms / 60_000
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("; ")
                    });
                    view.facts = facts(&[
                        (
                            "Blocker",
                            format!(
                                "pid {} · {} · {} · {}m",
                                root.pid,
                                root.user,
                                root.query,
                                root.duration_ms / 60_000
                            ),
                        ),
                        ("Waiting on it", waiting),
                        (
                            "Policy",
                            "cancel before terminate · terminate revalidates PID and query".into(),
                        ),
                        (
                            "Cancel effect",
                            "rolls back the blocking transaction · waiting sessions resume".into(),
                        ),
                    ]);
                    view.commands = vec![
                        format!("SELECT pg_cancel_backend({})", root.pid),
                        format!("SELECT pg_terminate_backend({})", root.pid),
                    ];
                    view.actions.push(if world.host.env.sensitive() {
                        UiAction::danger(ActionKey::CONFIRM, "Cancel blocker (pg_cancel_backend)")
                    } else {
                        UiAction::new(ActionKey::CONFIRM, "Cancel blocker (pg_cancel_backend)")
                    });
                    view.actions
                        .push(UiAction::new(TERMINATE, "Terminate (pg_terminate_backend)"));
                }
            }
            Intent::Clone { repo, login } => {
                let slug = repo.slug();
                let destination = format!("{}/{}", world.cwd, repo.name);
                view.title = format!("Clone {slug}?");
                view.facts = facts(&[
                    ("Account", format!("github.com/{login}")),
                    ("Owner", repo.owner.clone()),
                    ("Protocol", "ssh".into()),
                    ("Destination", destination.clone()),
                    ("Primary branch", repo.default_branch.clone()),
                    ("Fork", "no · direct clone".into()),
                    (
                        "Confirmation",
                        "asks once before creating the directory".into(),
                    ),
                ]);
                view.commands
                    .push(format!("gh repo clone {slug} {destination}"));
                view.actions
                    .push(UiAction::new(ActionKey::CONFIRM, "Clone (simulated)"));
            }
            Intent::Alias(action) => {
                view.title = format!("Alias {}", action.command);
                view.actions
                    .push(UiAction::new(ActionKey::CONFIRM, "Save alias"));
            }
        }
        if let Intent::Preview(action) = &view.intent {
            if let Some(host) = world
                .ssh
                .iter()
                .find(|host| action.intent_id() == Some(format!("ssh:{}", host.alias).as_str()))
            {
                view.title = format!("Connect to {}?", host.alias);
                view.width = 76;
                view.facts = facts(&[
                    ("Alias", format!("{} · literal, ~/.ssh/config", host.alias)),
                    ("HostName", host.host_name.clone()),
                    ("User", host.user.clone()),
                    (
                        "Port",
                        host.port
                            .map_or_else(|| "default 22".into(), |port| port.to_string()),
                    ),
                    (
                        "Identity file",
                        host.identity_file.as_ref().map_or_else(
                            || "none · agent or default keys".into(),
                            |file| format!("{file} · filename only, contents never read"),
                        ),
                    ),
                    ("Jump chain", host.chain()),
                    ("Host key policy", host.host_key_policy.label().into()),
                    (
                        "Multiplexing",
                        if host.multiplexed {
                            "active ControlMaster · reuses the connection"
                        } else {
                            "not multiplexed · new connection"
                        }
                        .into(),
                    ),
                ]);
                view.actions = vec![
                    UiAction::quiet(ActionKey::CANCEL, "Close"),
                    UiAction::new(ActionKey::CONFIRM, "Connect (simulated)"),
                ];
                view.ssh = Some(host.clone());
            }
        }
        if matches!(view.intent, Intent::Monitor) {
            view.width = 84;
        }
        view
    }
    fn props(&self) -> Dialog<'_> {
        let dialog = if matches!(self.intent, Intent::Alias(_)) {
            Dialog::prompt(DIALOG, &self.title, "Alias")
        } else {
            Dialog::new(DIALOG).title(&self.title)
        };
        let dialog = if self.description.is_empty() {
            dialog
        } else {
            dialog.description(&self.description)
        };
        dialog
            .actions(&self.actions)
            .cancel(ActionKey::CANCEL)
            .body_rows(
                self.facts
                    .len()
                    .saturating_add(usize::from(!self.commands.is_empty()))
                    .saturating_add(self.commands.len().min(8))
                    .min(usize::from(u16::MAX)) as u16,
            )
            .width(self.width)
    }
    pub(crate) fn open(&self, cx: &mut Cx<'_>) {
        let props = self.props();
        cx.open_layer(DIALOG, props.layer(cx));
        cx.focus(if matches!(self.intent, Intent::Alias(_)) {
            props.input_id()
        } else {
            props.action_id(0)
        });
    }
    pub(crate) fn poll(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<DialogAction>) {
        let mut state = std::mem::take(&mut self.dialog);
        let mut response = self.props().update(cx, &mut state);
        self.dialog = state;
        let lines: Vec<_> = self
            .commands
            .iter()
            .map(|line| ViewportLine::Plain(line))
            .collect();
        let output = TextViewport::new(CODE)
            .wrap(false)
            .update(cx, &mut self.output, &lines);
        let action = response.take_action();
        (response.erase() | output.erase(), action)
    }
    pub(crate) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        world: &mut World,
    ) -> (Response<()>, Option<Event>) {
        let (response, action) = self.poll(cx);
        let event = match action {
            Some(DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_)) => {
                Some(Event::Close)
            }
            Some(DialogAction::Action(key)) => Some(self.confirm(world, key)),
            None => None,
        };
        if event.is_some() {
            self.dialog.zeroize();
            cx.close_layer(DIALOG, Some(ActionKey::CLOSE));
        }
        (response, event)
    }
    fn confirm(&mut self, world: &mut World, key: ActionKey) -> Event {
        if self.context != Context::of(world) {
            return notice("Action context changed · review it again");
        }
        if let Some(reviewed) = &self.ssh {
            let mut targets = world.ssh.iter().filter(|host| host.alias == reviewed.alias);
            if targets.next() != Some(reviewed) || targets.next().is_some() {
                return notice("SSH target changed · review it again");
            }
        }
        match &mut self.intent {
            Intent::Quit => Event::Quit,
            Intent::Preview(action) => Event::Destination(dispatch::confirm_preview(world, action)),
            Intent::Trust { action, file } => {
                if catalogue::resolve_intent(world, action)
                    != Err(catalogue::IntentError::NeedsTrust)
                {
                    return notice("Task changed · review it again");
                }
                if world.trust_mise_file(file) {
                    notice(format!(
                        "Trusted {} · task re-resolved",
                        file.rsplit('/').next().unwrap_or(file)
                    ))
                } else {
                    notice("Task file changed · nothing trusted")
                }
            }
            Intent::Monitor => {
                notice("btm would take over the screen here · simulated handoff, holla stays")
            }
            Intent::Database { cancel, terminate } => {
                let review = if key == TERMINATE {
                    terminate.take()
                } else {
                    cancel.take()
                };
                match review {
                    Some(review) => match review.execute(world) {
                        Ok(report) => notice(report.to_string()),
                        Err(error) => notice(error.to_string()),
                    },
                    None => notice("Session review already consumed"),
                }
            }
            Intent::Clone { repo, login } => {
                if world.github.as_ref().is_some_and(|gh| {
                    gh.login == *login && gh.repos.iter().any(|current| current == repo)
                }) {
                    notice(format!(
                        "Would clone {} · simulated, nothing executed",
                        repo.slug()
                    ))
                } else {
                    notice("Clone target changed · review it again")
                }
            }
            Intent::Alias(action) => {
                match world.memory.set_alias(self.dialog.draft(), &action.command) {
                    Ok(()) => notice(format!(
                        "Alias {} → {}",
                        self.dialog.draft().trim(),
                        action.command
                    )),
                    Err(error) => notice(error.to_string()),
                }
            }
            Intent::About | Intent::Help => Event::Close,
        }
    }
    pub(crate) fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(DIALOG, |ui, area| {
            self.props().draw(ui, area, &self.dialog, |ui, body| {
                let facts: Vec<_> = self
                    .facts
                    .iter()
                    .map(|(label, value)| (label.as_str(), value.as_str()))
                    .collect();
                let height = self.facts.len().min(usize::from(u16::MAX)) as u16;
                Props::new(&facts).draw(
                    ui,
                    Rect::new(body.x, body.y, body.width, body.height.min(height)),
                );
                if !self.commands.is_empty() {
                    let lines: Vec<_> = self
                        .commands
                        .iter()
                        .map(|line| ViewportLine::Plain(line))
                        .collect();
                    TextViewport::new(CODE).wrap(false).draw(
                        ui,
                        Rect::new(
                            body.x,
                            body.y.saturating_add(height.saturating_add(1)),
                            body.width,
                            body.height.saturating_sub(height.saturating_add(1)),
                        ),
                        &self.output,
                        &lines,
                    );
                }
            })
        });
    }
}
fn facts(rows: &[(&str, String)]) -> Vec<(String, String)> {
    rows.iter()
        .map(|(label, value)| ((*label).into(), value.clone()))
        .collect()
}
fn notice(message: impl Into<String>) -> Event {
    Event::Destination(Destination::Notice(message.into()))
}
