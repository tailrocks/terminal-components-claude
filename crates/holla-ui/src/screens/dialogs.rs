use std::path::Path;

use super::context::Context;
use crate::dispatch::{self, Destination, TaskArgsStart};
use crate::domain::{
    action::{Action, Availability},
    github::{GhRepo, GhState},
    host::{Environment, HostKind},
    ssh::SshHost,
};
use crate::sim::{
    catalogue, pg,
    port_args::{PortArgsProducer, PortInvocationPlan, PortParseDisposition, PortReviewFacts},
    task_args::{InvocationPlan, SafeReviewFacts, TaskArgsProducer},
    world::World,
};
use termrock::{
    Action as UiAction, ActionKey, Cx, Dialog, DialogAction, DialogState, Field, FieldError, Id,
    ItemKey, Props, Rect, Response, Select, SelectAction, SelectField, SelectState, TextInput,
    TextInputState, TextViewport, Ui, ViewportLine, ViewportState,
};

pub(crate) const DIALOG: Id = Id::root("holla.dialog");
const CODE: Id = Id::root("holla.dialog.code");
const TERMINATE: ActionKey = ActionKey::application("holla.pg.terminate");
const BROWSE_FILES: ActionKey = ActionKey::application("holla.folder-browser.browse");
const EDIT_CLONE_OPTIONS: ActionKey = ActionKey::application("holla.clone.edit-options");
const SIMULATE_TASK_ARGS: ActionKey = ActionKey::application("holla.task-args.simulate");
const SIMULATE_PORT_ARGS: ActionKey = ActionKey::application("holla.port-args.simulate");
pub(crate) const CLONE_PROTOCOL: Id = Id::root("holla.clone.protocol");
pub(crate) const CLONE_DESTINATION: Id = Id::root("holla.clone.destination");
const CLONE_PROTOCOLS: [&str; 2] = ["SSH", "HTTPS"];
const EMPTY_DESTINATION: &str = "Enter a destination";
const CONTROL_DESTINATION: &str = "Destination cannot contain control characters";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloneProtocol {
    Ssh,
    Https,
}
impl CloneProtocol {
    const fn label(self) -> &'static str {
        match self {
            Self::Ssh => "ssh",
            Self::Https => "https",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Ssh => 0,
            Self::Https => 1,
        }
    }
}

#[derive(Clone)]
struct CloneOptions {
    protocol: CloneProtocol,
    destination: String,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CloneBinding {
    host: String,
    kind: HostKind,
    environment: Environment,
    cwd: String,
    effect_revision: u64,
}
impl CloneBinding {
    pub(crate) fn of(world: &World) -> Self {
        Self {
            host: world.host.name.clone(),
            kind: world.host.kind,
            environment: world.host.env,
            cwd: world.cwd.clone(),
            effect_revision: world.effect_revision,
        }
    }
}

#[derive(Clone)]
pub(crate) struct CloneRequest {
    action: Action,
    repo: GhRepo,
    github: GhState,
    login: String,
    cwd: String,
    binding: CloneBinding,
    options: CloneOptions,
}
impl CloneRequest {
    pub(crate) fn new(
        repo: GhRepo,
        login: String,
        action: Action,
        github: GhState,
        binding: CloneBinding,
    ) -> Self {
        let cwd = binding.cwd.clone();
        let destination = format!("{cwd}/{}", repo.name);
        Self {
            action,
            repo: repo.clone(),
            github,
            login,
            cwd,
            binding,
            options: CloneOptions {
                protocol: CloneProtocol::Ssh,
                destination,
            },
        }
    }

    pub(crate) fn target_is_current(&self, world: &World) -> bool {
        if self.binding != CloneBinding::of(world) || self.cwd != world.cwd {
            return false;
        }
        let Ok(current_action) = catalogue::resolve_intent(world, &self.action) else {
            return false;
        };
        current_action == self.action
            && current_action.intent_id() == Some("gh.clone")
            && world.github.as_ref() == Some(&self.github)
            && self.github.login == self.login
            && self.github.repos.iter().any(|repo| repo == &self.repo)
    }
}

struct CloneExecution {
    protocol: CloneProtocol,
    repository: String,
    destination: String,
}
impl CloneExecution {
    fn from_request(request: &CloneRequest) -> Self {
        Self {
            protocol: request.options.protocol,
            repository: request.repo.slug(),
            destination: bind_destination(&request.cwd, &request.options.destination),
        }
    }
}

fn bind_destination(cwd: &str, destination: &str) -> String {
    let cwd_prefix = format!("{}/", cwd.trim_end_matches('/'));
    if Path::new(destination).is_absolute()
        || destination == cwd
        || destination.starts_with(&cwd_prefix)
    {
        destination.to_owned()
    } else {
        Path::new(cwd)
            .join(destination)
            .to_string_lossy()
            .into_owned()
    }
}

pub(crate) fn validate_clone_destination(value: &str) -> Result<(), FieldError> {
    if value.trim().is_empty() {
        return Err(FieldError::new(EMPTY_DESTINATION));
    }
    if value.chars().any(|character| {
        matches!(
            character,
            '\0' | '\n' | '\r' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
        )
    }) {
        return Err(FieldError::new(CONTROL_DESTINATION));
    }
    Ok(())
}

fn destination_error_label(error: &FieldError) -> &'static str {
    if error.message.as_ref() == EMPTY_DESTINATION {
        EMPTY_DESTINATION
    } else {
        CONTROL_DESTINATION
    }
}

fn clone_destination_input<'a>(value: Option<&'a str>) -> TextInput<'a> {
    let input = TextInput::new(CLONE_DESTINATION)
        .placeholder("~/work/project/repository")
        .validate(&validate_clone_destination);
    match value {
        Some(value) => input.value(value),
        None => input,
    }
}

pub(crate) enum Intent {
    About,
    Quit,
    Monitor,
    Preview(Action),
    Trust {
        action: Action,
        file: String,
        args: Option<Box<TaskArgsStart>>,
    },
    PortArgsReview {
        producer: PortArgsProducer,
        plan: Option<Box<PortInvocationPlan>>,
        facts: PortReviewFacts,
    },
    TaskArgsReview {
        producer: TaskArgsProducer,
        plan: Option<Box<InvocationPlan>>,
        facts: SafeReviewFacts,
    },
    Database {
        cancel: Option<pg::Review>,
        terminate: Option<pg::Review>,
    },
    CloneReview(CloneRequest),
    CloneOptions(CloneRequest),
    Alias(Action),
    TopFiles,
    CleanupReview,
    FolderBrowser,
    HomeFind,
    CustomAction,
    SnapshotPage,
    DefinitionTrust,
}
pub(crate) enum Event {
    Close,
    BrowseFiles,
    Quit,
    Destination(Box<Destination>),
    OpenCloneOptions(CloneRequest),
    ReturnToCloneReview(CloneRequest),
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
    clone_protocol: CloneProtocol,
    clone_protocol_state: SelectState,
    clone_destination: String,
    clone_destination_state: TextInputState,
    clone_destination_error: Option<&'static str>,
}
impl ProductDialog {
    #[expect(
        clippy::too_many_lines,
        reason = "Pure intent-to-dialog projection keeps each pinned fact and action group together; effects live in the separate confirmation reducer"
    )]
    pub(crate) fn new(intent: Intent, world: &World) -> Self {
        let (clone_protocol, clone_destination) = match &intent {
            Intent::CloneReview(request) | Intent::CloneOptions(request) => (
                request.options.protocol,
                request.options.destination.clone(),
            ),
            _ => (CloneProtocol::Ssh, String::new()),
        };
        let mut clone_protocol_state = SelectState::default();
        clone_protocol_state.set_value(Some(ItemKey::index(clone_protocol.index())));
        let mut view = Self {
            intent,
            context: Context::of(world),
            ssh: None,
            width: 66,
            title: String::new(),
            description: String::new(),
            facts: Vec::new(),
            commands: Vec::new(),
            actions: vec![UiAction::quiet(ActionKey::CANCEL, "Close")],
            dialog: DialogState::default(),
            output: ViewportState::default(),
            clone_protocol,
            clone_protocol_state,
            clone_destination,
            clone_destination_state: TextInputState::default(),
            clone_destination_error: None,
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
                        "Junie-inspired Ratatui components (termrock)".into(),
                    ),
                ]);
            }
            Intent::Quit => {
                view.width = 54;
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
                    confirm_action("Quit"),
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
                    confirm_action(if matches!(action.availability, Availability::Ready) {
                        "Run (simulated)"
                    } else {
                        "Blocked"
                    })
                    .enabled(matches!(action.availability, Availability::Ready)),
                );
            }
            Intent::Trust { action, file, .. } => {
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
                view.actions.push(confirm_action("Trust file"));
            }
            Intent::PortArgsReview { facts: review, .. } => {
                view.title = "Review port lookup".into();
                view.description =
                    "Structured simulation only. No process will be launched or listener result inferred."
                        .into();
                let (disposition, warning) = match review.disposition {
                    PortParseDisposition::Parsed => ("parsed as u16", "none"),
                    PortParseDisposition::LegacyFallbackTo5173 => (
                        "legacy fallback to 5173",
                        "Invalid text: historical fallback",
                    ),
                };
                view.facts = facts(&[
                    (
                        "Action",
                        crate::screens::args::quote_value(&review.action_title),
                    ),
                    ("Host", crate::screens::args::quote_value(&review.host_name)),
                    ("Environment", format!("{:?}", review.environment)),
                    ("Input", disposition.into()),
                    ("Fallback warning", warning.into()),
                    ("Effective port", review.effective_port.to_string()),
                    ("Effect revision", review.effect_revision.to_string()),
                ]);
                view.commands = crate::screens::args::format_invocation_lines(&review.invocation);
                view.actions = vec![
                    UiAction::quiet(ActionKey::CANCEL, "Cancel"),
                    UiAction::new(SIMULATE_PORT_ARGS, "Simulate · nothing executed"),
                ];
            }
            Intent::TaskArgsReview { facts: review, .. } => {
                view.title = "Review task inputs".into();
                view.description =
                    "This is a structured simulation. No process will be launched.".into();
                view.facts = facts(&[
                    (
                        "Task",
                        crate::screens::args::quote_value(&review.action_title),
                    ),
                    ("Host", crate::screens::args::quote_value(&review.host_name)),
                    ("Environment", format!("{:?}", review.environment)),
                    ("Effect revision", review.effect_revision.to_string()),
                ]);
                view.commands = crate::screens::args::format_invocation_lines(&review.invocation);
                view.actions = vec![
                    UiAction::quiet(ActionKey::CANCEL, "Cancel"),
                    UiAction::new(SIMULATE_TASK_ARGS, "Simulate · nothing executed"),
                ];
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
                view.actions.push(confirm_action("Open btm (simulated)"));
            }
            Intent::Database { cancel, .. } => {
                view.width = 88;
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
                        confirm_action("Cancel blocker (pg_cancel_backend)")
                    });
                    view.actions
                        .push(UiAction::new(TERMINATE, "Terminate (pg_terminate_backend)"));
                }
            }
            Intent::CloneReview(request) => {
                let execution = CloneExecution::from_request(request);
                view.title = format!("Clone {}?", request.repo.slug());
                view.facts = facts(&[
                    ("Account", format!("github.com/{}", request.login)),
                    ("Owner", request.repo.owner.clone()),
                    ("Protocol", execution.protocol.label().into()),
                    ("Destination", execution.destination.clone()),
                    ("Primary branch", request.repo.default_branch.clone()),
                    ("Fork", "no · direct clone".into()),
                    (
                        "Confirmation",
                        "asks once before creating the directory".into(),
                    ),
                ]);
                view.actions.push(confirm_action("Clone (simulated)"));
                view.actions
                    .push(UiAction::new(EDIT_CLONE_OPTIONS, "Edit clone options"));
            }
            Intent::CloneOptions(_) => {
                view.width = 76;
                view.title = "Edit clone options".into();
                view.description =
                    "Only the transport and destination change; repository selection is retained."
                        .into();
                view.actions = vec![
                    UiAction::quiet(ActionKey::CANCEL, "Cancel"),
                    confirm_action("Review"),
                ];
            }
            Intent::Alias(action) => {
                view.width = 54;
                view.title = format!("Alias for “{}”", action.command);
                view.actions.push(confirm_action("Save alias"));
            }
            Intent::TopFiles => {
                view.title = "Largest files".into();
                view.facts = match world.disk.as_ref() {
                    Some(disk) => {
                        let top = disk
                            .candidates
                            .iter()
                            .max_by_key(|candidate| candidate.size_bytes);
                        vec![facts_row(
                            "Largest",
                            top.map_or_else(
                                || "none scanned".to_owned(),
                                |candidate| candidate.path.clone(),
                            ),
                        )]
                    }
                    None => vec![facts_row("Largest", "No disk scan is open".into())],
                };
            }
            Intent::CleanupReview => {
                view.title = "Review cleanup".into();
                let candidate = world.disk.as_ref().and_then(|disk| disk.candidates.first());
                view.facts = vec![
                    facts_row(
                        "Candidate",
                        candidate
                            .map_or_else(|| "none".to_owned(), |candidate| candidate.path.clone()),
                    ),
                    facts_row("Delete", "nothing is deleted from this review".into()),
                ];
            }
            Intent::FolderBrowser => {
                view.title = "Folder browser".into();
                view.facts = vec![facts_row("Directory", world.cwd.clone())];
                view.actions = vec![
                    UiAction::quiet(ActionKey::CANCEL, "Close"),
                    UiAction::new(BROWSE_FILES, "Browse"),
                ];
            }
            Intent::HomeFind => {
                view.title = "Find in home".into();
                view.facts = vec![facts_row("Query", "empty until this page is open".into())];
            }
            Intent::CustomAction => {
                view.title = "Custom action".into();
                view.facts = vec![facts_row("File", "no action file is loaded".into())];
            }
            Intent::SnapshotPage => {
                view.title = "Snapshot page".into();
                view.facts = vec![facts_row("Frame", "closed on the home frame".into())];
            }
            Intent::DefinitionTrust => {
                view.title = "Trust this definition".into();
                view.facts = vec![facts_row(
                    "Waiting",
                    "no untrusted definition is open".into(),
                )];
            }
        }
        if let Intent::Preview(action) = &view.intent
            && let Some(host) = world
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
                confirm_action("Connect (simulated)"),
            ];
            view.ssh = Some(host.clone());
        }
        if matches!(view.intent, Intent::About) {
            view.actions = vec![UiAction::new(ActionKey::CANCEL, "Close")];
        }
        if matches!(view.intent, Intent::Monitor) {
            view.width = 84;
        }
        view
    }
    fn props(&self) -> Dialog<'_> {
        let dialog = if matches!(self.intent, Intent::Alias(_)) {
            Dialog::prompt(DIALOG, &self.title, "Short name (e.g. gs)")
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
            .body_rows(if matches!(self.intent, Intent::CloneOptions(_)) {
                6
            } else {
                self.facts
                    .len()
                    .saturating_add(usize::from(!self.commands.is_empty()))
                    .saturating_add(self.commands.len().min(8))
                    .min(usize::from(u16::MAX)) as u16
            })
            .width(self.width)
    }
    pub(crate) fn is_editing(&self) -> bool {
        self.dialog.is_editing() || self.clone_destination_state.is_editing()
    }

    pub(crate) fn is_folder_browser(&self) -> bool {
        matches!(self.intent, Intent::FolderBrowser)
    }

    pub(crate) fn open(&self, cx: &mut Cx<'_>) {
        let props = self.props();
        cx.open_layer(DIALOG, props.layer(cx));
        cx.focus(if matches!(self.intent, Intent::CloneOptions(_)) {
            CLONE_DESTINATION
        } else if matches!(
            self.intent,
            Intent::PortArgsReview { .. } | Intent::TaskArgsReview { .. }
        ) {
            props.action_id(1)
        } else if matches!(self.intent, Intent::Alias(_)) {
            props.input_id()
        } else if matches!(
            self.intent,
            Intent::Database {
                cancel: Some(_),
                ..
            }
        ) {
            props.action_id(1)
        } else {
            props.action_id(0)
        });
    }
    pub(crate) fn poll(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<DialogAction>) {
        let mut response = Response::ignored();
        if matches!(self.intent, Intent::CloneOptions(_)) {
            let protocol = Select::new(CLONE_PROTOCOL).update(
                cx,
                &mut self.clone_protocol_state,
                &CLONE_PROTOCOLS,
            );
            if let Some(SelectAction::Chose(ItemKey::Index(index))) = protocol.action_ref() {
                self.clone_protocol = if *index == 1 {
                    CloneProtocol::Https
                } else {
                    CloneProtocol::Ssh
                };
            }
            response |= protocol.erase();

            let destination = clone_destination_input(None).update(
                cx,
                &mut self.clone_destination_state,
                &mut self.clone_destination,
            );
            self.clone_destination_error = self
                .clone_destination_state
                .error()
                .map(destination_error_label);
            response |= destination.erase();
        }

        let mut state = std::mem::take(&mut self.dialog);
        let mut dialog_response = self.props().update(cx, &mut state);
        self.dialog = state;
        let action = dialog_response.take_action();
        let lines: Vec<_> = self
            .commands
            .iter()
            .map(|line| ViewportLine::Plain(line))
            .collect();
        let output = code_viewport().update(cx, &mut self.output, &lines);
        response |= dialog_response.erase() | output.erase();
        (response, action)
    }
    pub(crate) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        world: &mut World,
    ) -> (Response<()>, Option<Event>) {
        let (response, action) = self.poll(cx);
        let event = match action {
            Some(DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_)) => {
                match &self.intent {
                    Intent::CloneOptions(request) => {
                        Some(Event::ReturnToCloneReview(request.clone()))
                    }
                    _ => Some(Event::Close),
                }
            }
            Some(DialogAction::Action(EDIT_CLONE_OPTIONS)) => match &self.intent {
                Intent::CloneReview(request) if request.target_is_current(world) => {
                    Some(Event::OpenCloneOptions(request.clone()))
                }
                Intent::CloneReview(_) => Some(notice("Clone target changed · review it again")),
                _ => Some(Event::Close),
            },
            Some(DialogAction::Action(ActionKey::CONFIRM))
                if matches!(self.intent, Intent::CloneOptions(_)) =>
            {
                self.accept_clone_options(world, cx)
            }
            Some(DialogAction::Action(key)) => Some(self.confirm(world, key)),
            None => None,
        };
        if event.is_some() {
            self.dialog.zeroize();
            cx.close_layer(DIALOG, Some(ActionKey::CLOSE));
        }

        let _ = confirm_action("Confirm");
        (response, event)
    }
    fn accept_clone_options(&mut self, world: &World, cx: &mut Cx<'_>) -> Option<Event> {
        let Intent::CloneOptions(request) = &self.intent else {
            return Some(Event::Close);
        };
        if !request.target_is_current(world) {
            return Some(notice("Clone target changed · review it again"));
        }
        if let Err(error) = validate_clone_destination(&self.clone_destination) {
            self.clone_destination_error = Some(destination_error_label(&error));
            cx.focus(CLONE_DESTINATION);
            return None;
        }
        let mut accepted = request.clone();
        accepted.options = CloneOptions {
            protocol: self.clone_protocol,
            destination: self.clone_destination.clone(),
        };
        Some(Event::ReturnToCloneReview(accepted))
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
            Intent::Preview(action) => {
                Event::Destination(Box::new(dispatch::confirm_preview(world, action)))
            }
            Intent::Trust { action, file, args } => {
                if catalogue::resolve_intent(world, action)
                    != Err(catalogue::IntentError::NeedsTrust)
                {
                    return notice("Task changed · review it again");
                }
                if let Some(mut continuation) = args.take() {
                    return match continuation.session.approve_trust(
                        world,
                        action,
                        &continuation.producer,
                        file,
                    ) {
                        Ok(action) => {
                            continuation.action = action;
                            Event::Destination(Box::new(Destination::TaskArgs(continuation)))
                        }
                        Err(error) => notice(error.to_string()),
                    };
                }
                if world.trust_mise_file(file) {
                    let acknowledgement = format!(
                        "Trusted {} · task re-resolved",
                        file.rsplit('/').next().unwrap_or(file)
                    );
                    match dispatch::invoke(world, action) {
                        Destination::Notice(message) => {
                            notice(format!("{acknowledgement} · {message}"))
                        }
                        destination => Event::Destination(Box::new(destination)),
                    }
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
                    Some(review)
                        if review.operation()
                            == if key == TERMINATE {
                                pg::Operation::Terminate
                            } else {
                                pg::Operation::Cancel
                            } =>
                    {
                        match review.execute(world) {
                            Ok(report) => notice(report.to_string()),
                            Err(error) => notice(error.to_string()),
                        }
                    }
                    Some(_) => notice("Session operation changed · review it again"),
                    None => notice("Session review already consumed"),
                }
            }
            Intent::CloneReview(request) if key == ActionKey::CONFIRM => {
                if !request.target_is_current(world) {
                    return notice("Clone target changed · review it again");
                }
                let execution = CloneExecution::from_request(request);
                notice(format!(
                    "Would clone {} via {} to {} · simulated, nothing executed",
                    execution.repository,
                    execution.protocol.label(),
                    execution.destination
                ))
            }
            Intent::PortArgsReview { producer, plan, .. } if key == SIMULATE_PORT_ARGS => {
                match plan.take() {
                    Some(plan) => Event::Destination(Box::new(dispatch::confirm_port_args(
                        world, producer, *plan,
                    ))),
                    None => notice("Port input review already consumed"),
                }
            }
            Intent::TaskArgsReview { producer, plan, .. } if key == SIMULATE_TASK_ARGS => {
                match plan.take() {
                    Some(plan) => Event::Destination(Box::new(dispatch::confirm_task_args(
                        world, producer, *plan,
                    ))),
                    None => notice("Task input review already consumed"),
                }
            }
            Intent::CloneReview(_) | Intent::CloneOptions(_) => Event::Close,
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
            Intent::FolderBrowser => {
                if key == BROWSE_FILES {
                    Event::BrowseFiles
                } else {
                    Event::Close
                }
            }
            Intent::About
            | Intent::TopFiles
            | Intent::CleanupReview
            | Intent::HomeFind
            | Intent::CustomAction
            | Intent::SnapshotPage
            | Intent::DefinitionTrust
            | Intent::PortArgsReview { .. }
            | Intent::TaskArgsReview { .. } => Event::Close,
        }
    }
    pub(crate) fn draw(&self, ui: &mut Ui<'_>) {
        let _ = confirm_action("Confirm");
        ui.layer(DIALOG, |ui, area| {
            self.props().draw(ui, area, &self.dialog, |ui, body| {
                let height = if matches!(self.intent, Intent::CloneOptions(_)) {
                    6
                } else {
                    self.facts.len().min(usize::from(u16::MAX)) as u16
                };
                if matches!(self.intent, Intent::CloneOptions(_)) {
                    let protocol = SelectField::new(Select::new(CLONE_PROTOCOL), &CLONE_PROTOCOLS);
                    let _ = Field::new("Protocol", protocol)
                        .required(true)
                        .plain(true)
                        .draw(
                            ui,
                            Rect::new(body.x, body.y, body.width, 3),
                            &self.clone_protocol_state,
                        );
                    let destination_y = body.y.saturating_add(3);
                    let _ = Field::new(
                        "Destination",
                        clone_destination_input(Some(&self.clone_destination)),
                    )
                    .required(true)
                    .plain(true)
                    .help("Relative destinations are based at the captured working directory")
                    .error(self.clone_destination_error)
                    .draw(
                        ui,
                        Rect::new(body.x, destination_y, body.width, 3),
                        &self.clone_destination_state,
                    );
                } else {
                    let facts: Vec<_> = self
                        .facts
                        .iter()
                        .map(|(label, value)| (label.as_str(), value.as_str()))
                        .collect();
                    Props::new(&facts).draw(
                        ui,
                        Rect::new(body.x, body.y, body.width, body.height.min(height)),
                    );
                }
                if !self.commands.is_empty() {
                    let lines: Vec<_> = self
                        .commands
                        .iter()
                        .map(|line| ViewportLine::Plain(line))
                        .collect();
                    code_viewport().draw(
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
            });
        });
    }
}
fn facts_row(label: &str, value: String) -> (String, String) {
    (label.to_owned(), value)
}
fn facts(rows: &[(&str, String)]) -> Vec<(String, String)> {
    rows.iter()
        .map(|(label, value)| ((*label).into(), value.clone()))
        .collect()
}
fn confirm_action(label: &'static str) -> UiAction<'static> {
    UiAction::new(ActionKey::CONFIRM, label)
}
fn code_viewport() -> TextViewport<'static> {
    TextViewport::new(CODE).wrap(false)
}
fn notice(message: impl Into<String>) -> Event {
    Event::Destination(Box::new(Destination::Notice(message.into())))
}
