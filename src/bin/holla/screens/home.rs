//! Home: the root surface. An always-armed query row over the ranked
//! priority stack — Suggested here / Recent here / Explore — with a reason
//! on every row, explicit scope on every nonlocal result, preview answering
//! CONCEPT.md §7, and structured alternatives (actions menu, clone review).

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{truncate, truncate_middle, width};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::segments::Segment;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use crate::domain::action::{Action, ActionKind, Availability, Risk, Scope};
use crate::domain::activity::{Activity, ActivityState};
use crate::domain::human_bytes;
use crate::domain::pg;
use crate::domain::ranking::Alias;
use crate::scenario::Scenario;
use crate::screens::modals::PickerModal;
use crate::screens::{Cx, Go, Modal, ModalResult, ModalTag, Screen};
use crate::sim::catalogue::{self, Section};
use crate::sim::plans;
use crate::sim::world::World;
use junie_tui::widgets::dialog::DialogBody;
use junie_tui::widgets::input::TextInput;

const QUERY: WidgetId = WidgetId::of("home.query");
const ROWS: WidgetId = WidgetId::of("home.row");
const PREVIEW: WidgetId = WidgetId::of("home.preview");
const TRUST: WidgetId = WidgetId::of("home.trust");
const ACTIONS: WidgetId = WidgetId::of("home.actions");
const CLONE_PICK: WidgetId = WidgetId::of("home.clone.pick");
const CLONE_REVIEW: WidgetId = WidgetId::of("home.clone.review");
const PG: WidgetId = WidgetId::of("home.pg");
const MONITOR: WidgetId = WidgetId::of("home.monitor");
const ALIAS: WidgetId = WidgetId::of("home.alias");

#[derive(Default)]
pub struct HomeScreen {
    pub query: String,
    /// Scope filter; `None` = all rings, priority stack decides order.
    pub scope: Option<Scope>,
}

impl HomeScreen {
    /// Visible sections after scope + query filtering.
    fn sections(&self, w: &World) -> Vec<Section> {
        catalogue::sections(w)
            .into_iter()
            .filter_map(|s| {
                let actions =
                    catalogue::visible(&s.actions, self.scope, &self.query, &w.memory, &w.cwd);
                (!actions.is_empty()).then_some(Section {
                    name: s.name,
                    actions,
                })
            })
            .collect()
    }

    /// Flattened visible rows, in render order.
    fn flat(&self, w: &World) -> Vec<Action> {
        self.sections(w)
            .into_iter()
            .flat_map(|s| s.actions)
            .collect()
    }

    fn focused_action(&self, w: &World, cx: &Cx) -> Option<Action> {
        let f = cx.focus.current()?;
        let flat = self.flat(w);
        (0..flat.len())
            .find(|&i| f == ROWS.child(i))
            .map(|i| flat[i].clone())
    }

    /// The action Enter/Ctrl+P/Ctrl+O applies to: focused row, else the top
    /// match while the query owns focus.
    fn subject(&self, w: &World, cx: &Cx) -> Option<Action> {
        let flat = self.flat(w);
        if let Some(a) = self.focused_action(w, cx) {
            return Some(a);
        }
        if cx.focus.is(QUERY) {
            flat.into_iter().next()
        } else {
            None
        }
    }

    /// Enter: dispatch by availability and risk. Risk changes treatment,
    /// never availability (§10) — broad actions open their review surface.
    /// Some flows own a dedicated surface: the pg blocking tree and the
    /// system snapshot with its btm handoff (§8.6).
    fn run(&self, a: &Action, w: &mut World, cx: &mut Cx) {
        if a.id == "pg.locks" {
            match pg_locks_dialog(w) {
                Some(d) => cx.open(Modal::Dialog(d), ModalTag::new("pg")),
                None => cx.status("No blockers · sessions healthy"),
            }
            return;
        }
        if a.id == "flow.monitor" {
            cx.open(Modal::Dialog(monitor_dialog(w)), ModalTag::new("monitor"));
            return;
        }
        match &a.availability {
            Availability::NeedsTrust(file) => {
                cx.open(
                    Modal::Dialog(trust_dialog(a, file)),
                    ModalTag::new("trust").key(file).n(row_of(a, w)),
                );
            }
            Availability::Blocked(why) => {
                cx.status(format!("{} · blocked: {why}", a.title));
            }
            Availability::Ready => match a.risk {
                Risk::ReadOnly if a.long_running => start_activity(a, w, cx),
                Risk::ReadOnly => simulate_run(a, w, cx),
                Risk::Bounded => {
                    cx.open(
                        Modal::Dialog(self.preview_for(a, w, true)),
                        ModalTag::new("preview").key(&a.id).n(row_of(a, w)),
                    );
                }
                Risk::Broad => match plans::plan_for(w, &a.id) {
                    Some(plan) => cx.go(Go::Plan(Box::new(plan))),
                    None => cx.status(format!("{} · no plan available", a.title)),
                },
            },
        }
    }

    /// The preview surface for an action: SSH connects resolve their alias
    /// chain first (§8.10); everything else answers §7 generically.
    fn preview_for(&self, a: &Action, w: &World, runnable: bool) -> Dialog {
        if let Some(alias) = a.id.strip_prefix("ssh:")
            && let Some(d) = ssh_resolution_dialog(w, alias)
        {
            return d;
        }
        preview_dialog(a, runnable)
    }

    /// Ctrl+P: the preview, answering §7 in facts form. Ready actions offer
    /// the run button; untrusted ones point at the trust gate.
    fn preview(&self, a: &Action, w: &World, cx: &mut Cx) {
        match &a.availability {
            Availability::NeedsTrust(file) => cx.open(
                Modal::Dialog(trust_dialog(a, file)),
                ModalTag::new("trust").key(file).n(row_of(a, w)),
            ),
            _ => cx.open(
                Modal::Dialog(self.preview_for(
                    a,
                    w,
                    matches!(a.availability, Availability::Ready),
                )),
                ModalTag::new("preview").key(&a.id).n(row_of(a, w)),
            ),
        }
    }

    /// Ctrl+O: structured alternatives for the subject action, including
    /// the ranking controls — pin, alias, hide, reset — each with its why.
    fn actions_menu(&self, a: &Action, w: &World, cx: &mut Cx) {
        let pinned = w.memory.pin_at(&w.cwd, &a.command);
        let aliased = w.memory.alias_for(&a.command).is_some();
        let hidden = w.memory.hidden_at(&w.cwd, &a.command);
        let items = vec![
            ("run".to_owned(), "Run".to_owned(), a.command.clone()),
            (
                "preview".to_owned(),
                "Preview".to_owned(),
                "facts before any run".to_owned(),
            ),
            (
                "copy".to_owned(),
                "Copy command".to_owned(),
                a.command.clone(),
            ),
            (
                "pin".to_owned(),
                if pinned { "Unpin here" } else { "Pin here" }.to_owned(),
                format!("ranking memory · {}", w.cwd),
            ),
            (
                "alias".to_owned(),
                if aliased {
                    "Change alias…"
                } else {
                    "Set alias…"
                }
                .to_owned(),
                "teach a short name · the query matches it".to_owned(),
            ),
            (
                "hide".to_owned(),
                if hidden { "Unhide here" } else { "Hide here" }.to_owned(),
                "gone from this folder's list · reset restores".to_owned(),
            ),
            (
                "reset".to_owned(),
                "Reset ranking".to_owned(),
                "clears pin, alias and hide for this command".to_owned(),
            ),
        ];
        cx.open(
            Modal::Custom(Box::new(PickerModal::new(ACTIONS, &a.title, items, false))),
            ModalTag::new("actions").key(&a.id).n(row_of(a, w)),
        );
    }

    /// Structured argument collection (§6.7): pick the repository, then
    /// review account/owner/protocol/destination/branch before the run.
    fn clone_pick(&self, w: &World, cx: &mut Cx) {
        let Some(gh) = &w.github else { return };
        let items = gh
            .repos
            .iter()
            .map(|r| {
                (
                    r.slug(),
                    r.slug(),
                    format!(
                        "{} · primary {}{}",
                        r.owner,
                        r.default_branch,
                        if r.private { " · private" } else { "" }
                    ),
                )
            })
            .collect();
        cx.open(
            Modal::Custom(Box::new(PickerModal::new(
                CLONE_PICK,
                "Clone which repository?",
                items,
                true,
            ))),
            ModalTag::new("clone.pick"),
        );
    }

    fn cycle_scope(&mut self, cx: &mut Cx) {
        self.scope = match self.scope {
            None => Some(Scope::Here),
            Some(Scope::Here) => Some(Scope::Project),
            Some(Scope::Project) => Some(Scope::Workspace),
            Some(Scope::Workspace) => Some(Scope::Host),
            Some(Scope::Host) => Some(Scope::Personal),
            Some(Scope::Personal) => None,
        };
        let label = self.scope.map_or("all", |s| s.label());
        cx.status(format!("Scope: {label}"));
        cx.focus.set(Some(QUERY));
    }
}

/// Row index of an action in the current flat list (modal-tag context).
fn row_of(a: &Action, w: &World) -> usize {
    catalogue::sections(w)
        .into_iter()
        .flat_map(|s| s.actions)
        .position(|x| x.id == a.id)
        .unwrap_or(0)
}

/// The §7 answers as facts: what happens, target, why, what changes,
/// freshness, confirmation. The command stays previewable as code.
fn preview_dialog(a: &Action, runnable: bool) -> Dialog {
    let facts = vec![
        Prop::new(
            "Will happen",
            format!("{} · in {}", a.command, a.workdir_label()),
        ),
        Prop::new("Target", format!("{} · {}", a.scope.label(), a.target)),
        Prop::new("Why recommended", &a.reason),
        Prop::new("Will change", a.changes()),
        Prop::new(
            "Freshness",
            "discovered at launch · simulated fixture, deterministic",
        ),
        Prop::new("Confirmation", a.risk.confirmation()),
    ];
    let confirm = if runnable {
        Button::primary(PREVIEW.sub("run"), "Run (simulated)")
    } else {
        Button::secondary(PREVIEW.sub("run"), "Blocked")
    };
    let d = Dialog::facts(
        PREVIEW,
        &a.title,
        facts,
        vec![a.command.clone()],
        None,
        confirm,
    );
    let close = Button::secondary(PREVIEW.sub("close"), "Close");
    let run = d.actions[1].clone();
    d.with_actions(vec![close, run], Some(0))
}

/// Trust gate for a mise task file: exact file, what it defines, the trust
/// scope. Runs nothing; flips trust so the task re-resolves as Ready.
fn trust_dialog(a: &Action, file: &str) -> Dialog {
    let facts = vec![
        Prop::new("Config file", file),
        Prop::new(
            "Defines",
            format!("{} → {}", a.id.trim_start_matches("task:"), a.command),
        ),
        Prop::new("Effect", "its tasks become runnable on this host"),
        Prop::new("Trust scope", "this exact file only · revocable"),
        Prop::new("Why asked", "mise refuses untrusted task files"),
    ];
    let d = Dialog::facts(
        TRUST,
        "Trust this task file?",
        facts,
        vec![a.command.clone()],
        None,
        Button::primary(TRUST.sub("trust"), "Trust file"),
    );
    let close = Button::secondary(TRUST.sub("close"), "Close");
    let trust = d.actions[1].clone();
    d.with_actions(vec![close, trust], Some(0))
}

/// Clone review (§12): account, owner, protocol, destination, primary
/// branch, fork behavior — before anything is created.
fn clone_review_dialog(w: &World, slug: &str) -> Option<Dialog> {
    let gh = w.github.as_ref()?;
    let repo = gh.repos.iter().find(|r| r.slug() == slug)?;
    let dest = format!("{}/{}", w.cwd, repo.name);
    let facts = vec![
        Prop::new("Account", format!("github.com/{}", gh.login)),
        Prop::new("Owner", &repo.owner),
        Prop::new("Protocol", "ssh"),
        Prop::new("Destination", &dest),
        Prop::new("Primary branch", &repo.default_branch),
        Prop::new("Fork", "no · direct clone"),
        Prop::new("Confirmation", "asks once before creating the directory"),
    ];
    let d = Dialog::facts(
        CLONE_REVIEW,
        &format!("Clone {slug}?"),
        facts,
        vec![format!("gh repo clone {slug} {dest}")],
        None,
        Button::primary(CLONE_REVIEW.sub("clone"), "Clone (simulated)"),
    );
    let close = Button::secondary(CLONE_REVIEW.sub("close"), "Close");
    let clone = d.actions[1].clone();
    Some(d.with_actions(vec![close, clone], Some(0)))
}

/// pg blocking tree (§8.7): the root blocker, who waits on it, and the
/// policy — cancel before terminate; terminate revalidates PID + query.
fn pg_locks_dialog(w: &World) -> Option<Dialog> {
    let sessions = w.pg.as_ref()?;
    let roots = pg::blockers(sessions);
    let root = roots.first()?;
    let blocked = pg::blocked_by(sessions, root.pid);
    let mins = |ms: u64| format!("{}m", ms / 60_000);
    let waiting = blocked
        .iter()
        .map(|s| format!("pid {} {} · {}", s.pid, s.user, mins(s.duration_ms)))
        .collect::<Vec<_>>()
        .join("; ");
    let facts = vec![
        Prop::new(
            "Blocker",
            format!(
                "pid {} · {} · {} · {}",
                root.pid,
                root.user,
                root.query,
                mins(root.duration_ms)
            ),
        ),
        Prop::new("Waiting on it", waiting),
        Prop::new(
            "Policy",
            "cancel before terminate · terminate revalidates PID and query",
        ),
        Prop::new(
            "Cancel effect",
            "rolls back the blocking transaction · waiting sessions resume",
        ),
    ];
    let code = vec![
        format!("SELECT pg_cancel_backend({})", root.pid),
        format!("SELECT pg_terminate_backend({})", root.pid),
    ];
    let mut cancel = Button::primary(PG.sub("cancel"), "Cancel blocker (pg_cancel_backend)");
    if w.host.env.sensitive() {
        cancel = Button::danger(PG.sub("cancel"), "Cancel blocker (pg_cancel_backend)");
    }
    let d = Dialog::facts(PG, "Database lock tree", facts, code, None, cancel);
    let close = Button::secondary(PG.sub("close"), "Close");
    let cancel = d.actions[1].clone();
    let terminate = Button::secondary(PG.sub("terminate"), "Terminate (pg_terminate_backend)");
    // the policy lines are the point of this dialog: give them room
    let mut d = d.with_actions(vec![close, cancel, terminate], Some(0));
    d.width = 88;
    Some(d)
}

/// SSH resolution preview (§8.10): what the literal alias expands to —
/// hostname, user, port, identity FILENAME (contents never read), jump
/// chain, host-key policy, multiplexing state — before any connection.
fn ssh_resolution_dialog(w: &World, alias: &str) -> Option<Dialog> {
    let h = w.ssh.iter().find(|h| h.alias == alias)?;
    let facts = vec![
        Prop::new("Alias", format!("{} · literal, ~/.ssh/config", h.alias)),
        Prop::new("HostName", &h.host_name),
        Prop::new("User", &h.user),
        Prop::new(
            "Port",
            h.port.map_or("default 22".to_owned(), |p| p.to_string()),
        ),
        Prop::new(
            "Identity file",
            h.identity_file
                .as_ref()
                .map_or("none · agent or default keys".to_owned(), |f| {
                    format!("{f} · filename only, contents never read")
                }),
        ),
        Prop::new("Jump chain", h.chain()),
        Prop::new("Host key policy", h.host_key_policy.label()),
        Prop::new(
            "Multiplexing",
            if h.multiplexed {
                "active ControlMaster · reuses the connection"
            } else {
                "not multiplexed · new connection"
            },
        ),
    ];
    let d = Dialog::facts(
        PREVIEW,
        &format!("Connect to {alias}?"),
        facts,
        vec![format!("ssh {}", h.alias)],
        None,
        Button::primary(PREVIEW.sub("run"), "Connect (simulated)"),
    );
    let close = Button::secondary(PREVIEW.sub("close"), "Close");
    let run = d.actions[1].clone();
    let mut d = d.with_actions(vec![close, run], Some(0));
    d.width = 76;
    Some(d)
}

/// System snapshot with the btm handoff (§8.6): a point-in-time view of
/// host vitals, then an honest handoff — btm owns live monitoring.
fn monitor_dialog(w: &World) -> Dialog {
    let m = &w.host.metrics;
    let (l1, l5, l15) = m.load_x100;
    let disk = w.disk.as_ref().map_or("not scanned".to_owned(), |d| {
        format!(
            "{}% used · {} of {}",
            d.used_percent(),
            human_bytes(d.used_bytes),
            human_bytes(d.total_bytes)
        )
    });
    let docker = w.docker.as_ref().map_or("not discovered".to_owned(), |d| {
        format!(
            "{} containers · {} running",
            d.containers.len(),
            d.running()
        )
    });
    let facts = vec![
        Prop::new("Host", w.host.identity()),
        Prop::new(
            "Load",
            format!(
                "{}.{:02} {}.{:02} {}.{:02}",
                l1 / 100,
                l1 % 100,
                l5 / 100,
                l5 % 100,
                l15 / 100,
                l15 % 100
            ),
        ),
        Prop::new(
            "Memory",
            format!("{} MB of {} MB", m.mem_used_mb, m.mem_total_mb),
        ),
        Prop::new("Disk /", disk),
        Prop::new("Docker", docker),
        Prop::new("Uptime", format!("{} days", m.uptime_days)),
        Prop::new(
            "Handoff",
            "btm takes over the screen for live monitoring · holla returns on exit",
        ),
    ];
    let d = Dialog::facts(
        MONITOR,
        "System snapshot",
        facts,
        vec!["btm".to_owned()],
        None,
        Button::primary(MONITOR.sub("btm"), "Open btm (simulated)"),
    );
    let close = Button::secondary(MONITOR.sub("close"), "Close");
    let btm = d.actions[1].clone();
    let mut d = d.with_actions(vec![close, btm], Some(0));
    d.width = 84;
    d
}

/// Alias prompt: a short name the query will match for this command.
/// Armed holla-side, same convention as the plan gate.
fn alias_prompt(a: &Action) -> Dialog {
    let mut d = Dialog::prompt(
        ALIAS,
        &format!("Alias for “{}”", a.command),
        TextInput::new(ALIAS.sub("name"), "Short name (e.g. gs)").plain_label(),
        "Save alias",
    );
    if let DialogBody::Input(i) = &mut d.body {
        i.begin_edit();
    }
    d
}

fn simulate_run(a: &Action, w: &mut World, cx: &mut Cx) {
    if w.scenario == Scenario::LaunchFailure && a.kind == ActionKind::Task {
        let mut act = Activity::new(
            next_activity_id(w),
            &a.title,
            &a.workdir_label(),
            ActivityState::Failed,
            w.now_ms(),
        );
        act.lines = vec![
            format!("$ {}", a.command),
            "test cache::stores ... FAILED".into(),
            "error: assertion failed · exit code 1".into(),
        ];
        w.activities.push(act);
        cx.error(format!("{} failed · exit code 1 · simulated", a.title));
        return;
    }
    cx.status(format!(
        "Would run: {} · simulated, nothing executed",
        a.command
    ));
}

/// Long-running work becomes a named activity that survives navigation.
/// `docker.logs` is the §12 follow-logs journey: one merged stream where
/// every line keeps its service identity (`api      | …`).
fn start_activity(a: &Action, w: &mut World, cx: &mut Cx) {
    if a.id == "docker.logs" {
        let lines = merged_log_lines(w);
        if lines.is_empty() {
            cx.status("No running containers to follow");
            return;
        }
        let mut act = Activity::new(
            next_activity_id(w),
            "container logs",
            &w.host.name.clone(),
            ActivityState::Running,
            w.now_ms(),
        );
        act.lines = lines;
        let id = act.id;
        w.activities.push(act);
        cx.status("Following container logs · merged stream".to_owned());
        cx.go(Go::Activity(id));
        return;
    }
    let mut act = Activity::new(
        next_activity_id(w),
        &a.title,
        &a.workdir_label(),
        ActivityState::Running,
        w.now_ms(),
    );
    act.lines = vec![format!("$ {}", a.command), "started · simulated".into()];
    w.activities.push(act);
    cx.status(format!("Activity started: {} · simulated", a.title));
}

/// Merged multi-service stream: running containers in name order, lines
/// interleaved round-robin, service prefix preserved per line. Fixture
/// samples are deterministic; times are virtual.
fn merged_log_lines(w: &World) -> Vec<String> {
    let Some(d) = &w.docker else { return vec![] };
    let mut names: Vec<&str> = d
        .containers
        .iter()
        .filter(|c| c.state == crate::domain::docker::ContainerState::Running)
        .map(|c| c.name.as_str())
        .collect();
    names.sort_unstable();
    let samples: Vec<Vec<&str>> = names.iter().map(|n| log_sample(n)).collect();
    let depth = samples.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = vec![];
    for round in 0..depth {
        for (i, name) in names.iter().enumerate() {
            if let Some(l) = samples[i].get(round) {
                out.push(format!("{name:<8} | {l}"));
            }
        }
    }
    out
}

/// Deterministic log lines per known service; unknown services get a
/// generic heartbeat so the stream is never empty for a running container.
fn log_sample(name: &str) -> Vec<&'static str> {
    match name {
        "api" => vec![
            "10:24:01 GET /health 200 · 2ms",
            "10:24:03 POST /orders 201 · 41ms",
            "10:24:09 GET /health 200 · 1ms",
            "10:24:12 GET /orders/9918 200 · 6ms",
        ],
        "worker" => vec![
            "10:24:02 job 1148 done · 310ms",
            "10:24:06 job 1149 done · 288ms",
            "10:24:10 job 1150 running",
        ],
        "redis" => vec![
            "10:24:01 3 clients connected",
            "10:24:05 background save done",
            "10:24:11 keyspace hits 10422 misses 91",
        ],
        "postgres" => vec![
            "10:24:02 checkpoint complete",
            "10:24:08 autovacuum payments",
        ],
        "nginx" => vec!["10:24:03 200 GET /", "10:24:07 304 GET /assets/app.css"],
        "gitea" => vec!["10:24:04 GET /api/v1/version 200"],
        _ => vec!["10:24:00 alive", "10:24:06 alive"],
    }
}

fn next_activity_id(w: &World) -> u32 {
    w.activities.iter().map(|a| a.id).max().unwrap_or(0) + 1
}

trait ActionExt {
    fn workdir_label(&self) -> String;
}

impl ActionExt for Action {
    fn workdir_label(&self) -> String {
        if self.workdir.is_empty() {
            self.target.clone()
        } else {
            self.workdir.clone()
        }
    }
}

impl Screen for HomeScreen {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if key.ctrl_char('s') {
            self.cycle_scope(cx);
            return Outcome::Changed;
        }
        if key.ctrl_char('p') {
            if let Some(a) = self.subject(w, cx) {
                if a.id == "gh.clone" {
                    self.clone_pick(w, cx);
                } else {
                    self.preview(&a, w, cx);
                }
            }
            return Outcome::Changed;
        }
        if key.ctrl_char('o') {
            if let Some(a) = self.subject(w, cx) {
                self.actions_menu(&a, w, cx);
            }
            return Outcome::Changed;
        }
        match key.code {
            KeyCode::Char(c) if key.plain() => {
                // the query is always armed; a bare `?`/`q` on an empty query
                // are the chrome keys instead of text
                if self.query.is_empty() && c == '?' {
                    cx.help();
                    return Outcome::Changed;
                }
                if self.query.is_empty() && c == 'q' {
                    cx.go(Go::Quit);
                    return Outcome::Changed;
                }
                self.query.push(c);
                Outcome::Changed
            }
            KeyCode::Backspace => {
                if self.query.pop().is_some() {
                    Outcome::Changed
                } else {
                    Outcome::Ignored
                }
            }
            KeyCode::Down => {
                cx.focus.next(cx.ring);
                Outcome::Changed
            }
            KeyCode::Up => {
                cx.focus.prev(cx.ring);
                Outcome::Changed
            }
            KeyCode::Enter => match self.subject(w, cx) {
                Some(a) if a.id == "gh.clone" => {
                    self.clone_pick(w, cx);
                    Outcome::Changed
                }
                Some(a) => {
                    self.run(&a, w, cx);
                    Outcome::Changed
                }
                None => Outcome::Consumed,
            },
            KeyCode::Esc => {
                if self.scope.is_some() {
                    self.scope = None;
                    cx.status("Scope: all");
                    Outcome::Changed
                } else if !self.query.is_empty() {
                    self.query.clear();
                    Outcome::Changed
                } else {
                    Outcome::Ignored
                }
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, _pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        let flat = self.flat(w);
        for (i, a) in flat.iter().enumerate() {
            if id == ROWS.child(i) {
                let a = a.clone();
                self.run(&a, w, cx);
                return Outcome::Changed;
            }
        }
        Outcome::Ignored
    }

    fn on_modal(
        &mut self,
        tag: &ModalTag,
        result: ModalResult,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        match (tag.kind, result) {
            (
                "preview",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                if let Some(a) = catalogue::catalogue(w)
                    .into_iter()
                    .find(|a| a.id == tag.key)
                {
                    match a.risk {
                        Risk::ReadOnly if a.long_running => start_activity(&a, w, cx),
                        Risk::ReadOnly | Risk::Bounded => simulate_run(&a, w, cx),
                        Risk::Broad => {
                            if let Some(plan) = plans::plan_for(w, &a.id) {
                                cx.go(Go::Plan(Box::new(plan)));
                            }
                        }
                    }
                }
                Outcome::Changed
            }
            (
                "trust",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                if w.trust_mise_file(&tag.key) {
                    cx.status(format!(
                        "Trusted {} · task re-resolved",
                        tag.key.rsplit('/').next().unwrap_or(&tag.key)
                    ));
                }
                Outcome::Changed
            }
            ("actions", ModalResult::Custom(payload)) => {
                let Some(a) = catalogue::catalogue(w)
                    .into_iter()
                    .find(|a| a.id == tag.key)
                else {
                    return Outcome::Ignored;
                };
                match payload.as_str() {
                    "run" => self.run(&a, w, cx),
                    "preview" => self.preview(&a, w, cx),
                    "copy" => cx.copy(a.command.clone()),
                    "pin" => {
                        if w.memory.pin_at(&w.cwd, &a.command) {
                            w.memory
                                .pins
                                .retain(|p| p.command != a.command || p.path != w.cwd);
                            cx.status(format!("Unpinned {}", a.command));
                        } else {
                            w.memory.pins.push(crate::domain::ranking::Pin {
                                path: w.cwd.clone(),
                                command: a.command.clone(),
                            });
                            cx.status(format!("Pinned {} here", a.command));
                        }
                    }
                    "alias" => {
                        let d = alias_prompt(&a);
                        cx.open(Modal::Dialog(d), ModalTag::new("alias").key(&a.command));
                    }
                    "hide" => {
                        if w.memory.hidden_at(&w.cwd, &a.command) {
                            w.memory
                                .hides
                                .retain(|h| h.command != a.command || h.path != w.cwd);
                            cx.status(format!("Unhidden {} here", a.command));
                        } else {
                            w.memory.hides.push(crate::domain::ranking::Pin {
                                path: w.cwd.clone(),
                                command: a.command.clone(),
                            });
                            cx.status(format!(
                                "Hidden {} here · Reset ranking restores",
                                a.command
                            ));
                        }
                    }
                    "reset" => {
                        w.memory.reset_at(&w.cwd, &a.command);
                        cx.status(format!(
                            "Reset ranking for {} · pins, aliases, hides cleared",
                            a.command
                        ));
                    }
                    _ => {}
                }
                Outcome::Changed
            }
            (
                "alias",
                ModalResult::Dialog {
                    action: Some(1),
                    text: Some(name),
                },
            ) => {
                let name = name.trim();
                if name.is_empty() || name.chars().any(char::is_whitespace) {
                    cx.status("Alias needs one word · nothing saved".to_owned());
                    return Outcome::Changed;
                }
                w.memory.aliases.push(Alias {
                    alias: name.to_owned(),
                    expansion: tag.key.clone(),
                });
                cx.status(format!("Alias {name} → {}", tag.key));
                Outcome::Changed
            }
            (
                "pg",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                // cancel the root blocker: fixture rolls the transaction back
                // and every session waiting on it resumes.
                let Some(sessions) = &mut w.pg else {
                    return Outcome::Ignored;
                };
                let Some(root) = pg::blockers(sessions).first().map(|s| s.pid) else {
                    cx.status("No blockers · sessions healthy".to_owned());
                    return Outcome::Changed;
                };
                sessions.retain(|s| s.pid != root);
                let resumed = sessions
                    .iter()
                    .filter(|s| s.blocked_by == Some(root))
                    .count();
                for s in sessions.iter_mut() {
                    if s.blocked_by == Some(root) {
                        s.blocked_by = None;
                    }
                }
                cx.status(format!(
                    "Cancelled pid {root} · {resumed} waiting sessions resumed"
                ));
                Outcome::Changed
            }
            (
                "pg",
                ModalResult::Dialog {
                    action: Some(2), ..
                },
            ) => {
                // terminate revalidates: if the pid already left (cancelled
                // elsewhere, or its transaction finished), nothing happens.
                let Some(sessions) = &mut w.pg else {
                    return Outcome::Ignored;
                };
                let Some(root) = pg::blockers(sessions).first().map(|s| s.pid) else {
                    cx.status(
                        "Revalidated: pid no longer exists · nothing to terminate".to_owned(),
                    );
                    return Outcome::Changed;
                };
                sessions.retain(|s| s.pid != root);
                let resumed = sessions
                    .iter()
                    .filter(|s| s.blocked_by == Some(root))
                    .count();
                for s in sessions.iter_mut() {
                    if s.blocked_by == Some(root) {
                        s.blocked_by = None;
                    }
                }
                cx.status(format!(
                    "Terminated pid {root} after revalidation · {resumed} waiting sessions resumed"
                ));
                Outcome::Changed
            }
            (
                "monitor",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                cx.status(
                    "btm would take over the screen here · simulated handoff, holla stays"
                        .to_owned(),
                );
                Outcome::Changed
            }
            ("clone.pick", ModalResult::Custom(slug)) => {
                if let Some(d) = clone_review_dialog(w, &slug) {
                    cx.open(Modal::Dialog(d), ModalTag::new("clone.review").key(&slug));
                }
                Outcome::Changed
            }
            (
                "clone.review",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                cx.status(format!(
                    "Would clone {} · simulated, nothing executed",
                    tag.key
                ));
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        fill(buf, area, t.base());
        if area.height < 4 {
            return;
        }
        // query row: field plane, focus wedge, placeholder or underlined text
        let field = Rect::new(area.x, area.y, area.width, 1);
        let fs = t.field_style(ctx.state(QUERY));
        fill(buf, field, fs);
        ctx.control(QUERY, field, false);
        buf.set_string(
            field.x,
            field.y,
            "▎",
            Style::new().fg(t.focus).bg(fs.bg.unwrap_or(t.field)),
        );
        let text_x = field.x + 2;
        let scope_w = self.scope.map_or(0, |s| s.label().len() as u16 + 10);
        let text_w = field.width.saturating_sub(3 + scope_w) as usize;
        if self.query.is_empty() {
            buf.set_string(
                text_x,
                field.y,
                truncate("Type to filter · actions, files, hosts", text_w),
                fs.fg(t.text_muted),
            );
        } else {
            buf.set_string(
                text_x,
                field.y,
                truncate(&self.query, text_w),
                fs.add_modifier(Modifier::UNDERLINED)
                    .underline_color(t.accent),
            );
        }
        if let Some(s) = self.scope {
            let label = format!("scope: {}", s.label());
            buf.set_string(
                field.right().saturating_sub(label.len() as u16 + 1),
                field.y,
                &label,
                Style::new()
                    .fg(t.tone(Tone::Secondary))
                    .bg(fs.bg.unwrap_or(t.field)),
            );
        }
        ctx.set_cursor(Position::new(
            text_x + width(&self.query).min(text_w) as u16,
            field.y,
        ));

        // result sections
        let mut y = area.y + 2;
        let mut n = 0usize;
        let sections = self.sections(w);
        if sections.is_empty() {
            let note = if self.query.trim().is_empty() {
                match self.scope {
                    Some(s) => format!("Nothing in scope {} here", s.label()),
                    None => "Nothing discovered here yet".to_owned(),
                }
            } else {
                format!("Nothing matches “{}” here", self.query.trim())
            };
            buf.set_string(
                area.x + 2,
                y,
                truncate(&note, area.width.saturating_sub(4) as usize),
                t.muted(),
            );
        }
        for (si, s) in sections.iter().enumerate() {
            if si > 0 {
                y += 1;
            }
            if y >= area.bottom() {
                break;
            }
            buf.set_string(
                area.x + 2,
                y,
                truncate(s.name, area.width.saturating_sub(4) as usize),
                t.faint(),
            );
            y += 1;
            for a in &s.actions {
                if y >= area.bottom() {
                    break;
                }
                let id = ROWS.child(n);
                let row = Rect::new(area.x, y, area.width, 1);
                ctx.control(id, row, false);
                let st = t.row(ctx.state(id), t.canvas);
                fill(buf, row, st);
                buf.set_string(row.x, y, "▎", t.gutter(ctx.state(id), t.canvas, false));
                // title (+ warning marker), then scope tag, then right reason
                let mut x = row.x + 2;
                let title_w = width(&a.title) as u16;
                if x + title_w < row.right() {
                    buf.set_string(x, y, &a.title, st);
                    x += title_w;
                } else {
                    // even alone the title overflows: ellipsize, never guillotine
                    let budget = (row.right() - x).saturating_sub(1) as usize;
                    buf.set_string(x, y, truncate(&a.title, budget), st);
                    x = row.right();
                }
                if matches!(
                    a.availability,
                    Availability::NeedsTrust(_) | Availability::Blocked(_)
                ) && x + 2 < row.right()
                {
                    buf.set_string(
                        x + 1,
                        y,
                        "▲",
                        Style::new()
                            .fg(t.tone(Tone::Warning))
                            .bg(st.bg.unwrap_or(t.canvas)),
                    );
                    x += 2;
                }
                // reason owns the right edge only with a real two-cell gap
                // after the title; otherwise it yields the row (title and
                // scope carry the meaning) — never a flush collision
                let rw = width(&a.reason) as u16;
                let reason_x = if row.width > rw + 24 && row.right() > x + 2 + rw {
                    row.right().saturating_sub(rw + 1)
                } else {
                    row.right()
                };
                if a.scope != Scope::Here {
                    let tag = if a.scope_label.is_empty() {
                        format!("· {}", a.scope.label())
                    } else {
                        format!("· {} {}", a.scope.label(), a.scope_label)
                    };
                    let budget = reason_x.saturating_sub(x + 3) as usize;
                    if budget >= 10 {
                        buf.set_string(
                            x + 1,
                            y,
                            truncate_middle(&tag, budget),
                            Style::new()
                                .fg(t.tone(Tone::Faint))
                                .bg(st.bg.unwrap_or(t.canvas)),
                        );
                    }
                }
                if reason_x < row.right() {
                    buf.set_string(
                        reason_x,
                        y,
                        &a.reason,
                        Style::new()
                            .fg(t.tone(Tone::Muted))
                            .bg(st.bg.unwrap_or(t.canvas)),
                    );
                }
                y += 1;
                n += 1;
            }
        }
        // discovery honesty: what is still scanning, what failed
        if y + 1 < area.bottom() {
            let pending: Vec<&str> = w
                .discovery
                .iter()
                .filter(|d| !d.done && !d.failed)
                .map(|d| d.domain.label())
                .collect();
            if !pending.is_empty() {
                buf.set_string(
                    area.x + 2,
                    area.bottom() - 1,
                    format!("Scanning this folder… ({})", pending.join(", ")),
                    t.faint(),
                );
            } else {
                let failed: Vec<&str> = w
                    .discovery
                    .iter()
                    .filter(|d| d.failed)
                    .map(|d| d.domain.label())
                    .collect();
                if !failed.is_empty() {
                    buf.set_string(
                        area.x + 2,
                        area.bottom() - 1,
                        format!("▲ {}: discovery failed", failed.join(", ")),
                        Style::new().fg(t.tone(Tone::Warning)),
                    );
                }
            }
        }
    }

    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint> {
        let strip = w
            .activities
            .is_empty()
            .then_some(vec![])
            .unwrap_or_else(|| vec![hint("Ctrl+A", "Activities")]);
        if focus.is_some_and(|f| (0..24).any(|i| f == ROWS.child(i))) {
            let mut h = vec![
                hint("Enter", "Run"),
                hint("Ctrl+P", "Preview"),
                hint("Ctrl+O", "Actions"),
                hint("↑", "Query"),
                hint("Esc", "Clear"),
            ];
            h.extend(strip);
            h
        } else {
            let mut h = vec![
                hint("Type", "Filter"),
                hint("↓", "Results"),
                hint("Enter", "Run top match"),
                hint("Ctrl+S", "Scope"),
                hint("F10", "Menu"),
                hint("q", "Quit"),
            ];
            h.extend(strip);
            h
        }
    }

    fn crumb(&self, w: &World) -> String {
        w.cwd.clone()
    }

    fn header_right(&self, _w: &World) -> Vec<Segment> {
        match self.scope {
            Some(s) => {
                vec![Segment::new(format!("scope: {}", s.label()), Tone::Secondary).priority(6)]
            }
            None => vec![],
        }
    }

    fn is_editing(&self) -> bool {
        // the query is always armed: plain characters filter, chrome uses
        // chords and function keys
        true
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(QUERY)
    }

    fn on_esc_top(&mut self, _w: &mut World, _cx: &mut Cx) -> Outcome {
        // home is the base surface: Esc never leaves it
        Outcome::Consumed
    }
}
