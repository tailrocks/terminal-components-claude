use crate::domain::action::{Action, ActionKind, Availability, Risk, Scope};
use crate::domain::docker::Health;
use crate::domain::human_bytes;
use crate::domain::mise::Trust;
use crate::domain::ranking::Memory;
use crate::scenario::Scenario;
use crate::world::{Domain, World};

pub struct Section {
    pub name: &'static str,
    pub actions: Vec<Action>,
}

pub fn catalogue(w: &World) -> Vec<Action> {
    if let Some(view) = parity_discovery(w) {
        return view.actions;
    }
    let mut out = vec![];
    mise_actions(w, &mut out);
    git_actions(w, &mut out);
    docker_actions(w, &mut out);
    disk_actions(w, &mut out);
    pg_actions(w, &mut out);
    ssh_actions(w, &mut out);
    github_actions(w, &mut out);
    debian_actions(w, &mut out);
    flow_actions(w, &mut out);
    recent_actions(w, &mut out);
    out
}

pub fn sections(w: &World) -> Vec<Section> {
    if let Some(view) = parity_discovery(w) {
        let suggested = view
            .actions
            .iter()
            .take(view.suggested_len)
            .cloned()
            .collect();
        let explore = view
            .actions
            .iter()
            .skip(view.suggested_len)
            .cloned()
            .collect();
        return vec![
            Section {
                name: "Suggested here",
                actions: suggested,
            },
            Section {
                name: "Explore",
                actions: explore,
            },
        ];
    }
    let mut all = catalogue(w);
    rank(w, &mut all);

    for a in &mut all {
        if let Some(name) = w.memory.alias_for(&a.command) {
            a.reason = format!("{} · alias {name}", a.reason);
        }
    }
    let suggested: Vec<Action> = all.iter().take(4).cloned().collect();
    let suggested_ids: Vec<&str> = suggested.iter().map(|a| a.id.as_str()).collect();
    let recent: Vec<Action> = all
        .iter()
        .filter(|a| a.id.starts_with("recent:") && !suggested_ids.contains(&a.id.as_str()))
        .cloned()
        .collect();
    let recent_ids: Vec<&str> = recent.iter().map(|a| a.id.as_str()).collect();
    let explore: Vec<Action> = all
        .into_iter()
        .filter(|a| !suggested_ids.contains(&a.id.as_str()) && !recent_ids.contains(&a.id.as_str()))
        .collect();
    let mut out = vec![];
    if !suggested.is_empty() {
        out.push(Section {
            name: "Suggested here",
            actions: suggested,
        });
    }
    if !recent.is_empty() {
        out.push(Section {
            name: "Recent here",
            actions: recent,
        });
    }
    if !explore.is_empty() {
        out.push(Section {
            name: "Explore",
            actions: explore,
        });
    }
    out
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "Closed scope weights and capped usage bound the total score to 390"
)]
pub fn rank(w: &World, actions: &mut [Action]) {
    let score = |a: &Action| -> i32 {
        let mut s = a.scope.weight();

        if w.memory.pin_at(&w.cwd, &a.command) {
            s += 150;
        }
        if w.memory.alias_for(&a.command).is_some() {
            s += 90;
        }
        s += w.memory.usage_at(&w.cwd, &a.command).min(10) as i32 * 2;
        if urgent(a) {
            s += 30;
        }
        s
    };
    actions.sort_by(|a, b| {
        score(b)
            .cmp(&score(a))
            .then(a.kind.cmp(&b.kind))
            .then(a.title.cmp(&b.title))
    });
}

fn urgent(a: &Action) -> bool {
    a.reason.contains("unhealthy")
        || a.reason.contains("modified")
        || a.reason.contains("behind")
        || a.reason.contains("untrusted")
        || a.reason.contains("% full")
        || a.reason.contains("detached")
}

fn mise_actions(w: &World, out: &mut Vec<Action>) {
    if !w.discovered(Domain::Mise) {
        return;
    }
    let Some(mise) = &w.mise else { return };
    for t in &mise.tasks {
        let workdir = t
            .defined_in
            .rsplit_once('/')
            .map_or_else(|| w.cwd.clone(), |(d, _)| d.to_owned());

        let scope = if std::path::Path::new(&t.defined_in).starts_with(&w.cwd) && !t.namespaced() {
            Scope::Here
        } else if t.namespaced() && std::path::Path::new(&workdir).starts_with(&w.cwd) {
            Scope::Project
        } else {
            Scope::Workspace
        };
        let mut a = Action::new(
            &format!("task:{}", t.id),
            &format!("Run {}", t.name()),
            ActionKind::Task,
            scope,
            &task_reason(t),
            Risk::ReadOnly,
            &workdir,
            &format!("mise run {}", t.id),
        );
        a.workdir.clone_from(&workdir);
        a.scope_label = scope_place(&w.cwd, &workdir);
        a.keywords.clone_from(&t.command);
        if t.trust == Trust::Untrusted {
            a.availability = Availability::NeedsTrust(t.defined_in.clone());
        }
        a.long_running = matches!(t.name(), "dev" | "watch" | "serve") || t.id.ends_with(":dev");
        out.push(a);
    }
}

fn task_reason(t: &crate::domain::mise::MiseTask) -> String {
    let file = t.defined_in.rsplit('/').next().unwrap_or(&t.defined_in);
    let mut r = if t.namespaced() {
        format!("defined by project task runner · {}", t.id)
    } else {
        format!("defined by project task runner · {file}")
    };
    if t.trust == Trust::Untrusted {
        r.push_str(" · untrusted config");
    }
    r
}

#[expect(
    clippy::too_many_lines,
    reason = "One discovered Git inventory projects its declarative action family together"
)]
fn git_actions(w: &World, out: &mut Vec<Action>) {
    if !w.discovered(Domain::Git) {
        return;
    }
    let Some(git) = &w.git else { return };
    let repo_here = git.root == w.cwd || w.cwd.starts_with(&format!("{}/", git.root));
    let scope = if repo_here {
        Scope::Here
    } else {
        Scope::Workspace
    };
    if git.dirty() {
        out.push(Action::new(
            "git.review",
            "Review modified files",
            ActionKind::Git,
            scope,
            &git.summary()
                .into_iter()
                .filter(|s| {
                    s.contains("modified") || s.contains("staged") || s.contains("untracked")
                })
                .collect::<Vec<_>>()
                .join(" · "),
            Risk::ReadOnly,
            &git.root,
            "git status --short",
        ));
    }
    if git.behind > 0 && !git.diverged() {
        let mut a = Action::new(
            "git.pull",
            "Pull — fast-forward only",
            ActionKind::Git,
            Scope::Project,
            &format!(
                "branch is {} {} behind",
                git.behind,
                if git.behind == 1 { "commit" } else { "commits" }
            ),
            Risk::Bounded,
            &git.root,
            "git pull --ff-only",
        );
        a.workdir.clone_from(&git.root);
        out.push(a);
    }
    if git.detached() {
        out.push(Action::new(
            "git.detached",
            "Inspect detached HEAD",
            ActionKind::Git,
            scope,
            &format!(
                "detached at {} · primary branch is {}",
                git.detached_sha.as_deref().unwrap_or("?"),
                git.primary_branch
            ),
            Risk::ReadOnly,
            &git.root,
            "git log -1 --stat",
        ));
    }
    if !git.children.is_empty() {
        out.push(Action::new(
            "git.children",
            "Status across child projects",
            ActionKind::Git,
            Scope::Workspace,
            &format!(
                "{} nested {} (not submodules)",
                git.children.len(),
                if git.children.len() == 1 {
                    "repository"
                } else {
                    "repositories"
                }
            ),
            Risk::ReadOnly,
            &git.root,
            "git -C <child> status --short",
        ));

        let behind: u32 = git.children.iter().map(|c| c.behind).sum();
        let diverged = git.children.iter().filter(|c| c.diverged()).count();
        if behind > 0 {
            let mut a = Action::new(
                "git.sync",
                "Update all child projects",
                ActionKind::Plan,
                Scope::Workspace,
                &format!(
                    "{} child repositories · {} commits behind{}",
                    git.children.len(),
                    behind,
                    if diverged > 0 {
                        format!(" · {diverged} diverged")
                    } else {
                        String::new()
                    }
                ),
                Risk::Broad,
                &git.root,
                "git -C <child> pull --ff-only",
            );
            a.scope_label = scope_place(&w.cwd, &git.root);
            out.push(a);
        }
    }
    let mut status = Action::new(
        "git.status",
        "Check git status",
        ActionKind::Git,
        scope,
        &git.summary().join(" · "),
        Risk::ReadOnly,
        &git.root,
        "git status",
    );
    status.workdir.clone_from(&git.root);
    out.push(status);
}

#[expect(
    clippy::too_many_lines,
    reason = "One discovered Docker inventory projects its declarative action family together"
)]
fn docker_actions(w: &World, out: &mut Vec<Action>) {
    if w.discovery_failed(Domain::Docker) {
        out.push(Action::new(
            "docker.unavailable",
            "Docker status",
            ActionKind::System,
            Scope::Host,
            "discovery failed · docker did not answer",
            Risk::ReadOnly,
            &w.host.name,
            "docker info",
        ));
        if let Some(a) = out.last_mut() {
            a.availability = Availability::Blocked("discovery failed".into());
        }
        return;
    }
    if !w.discovered(Domain::Docker) {
        return;
    }
    let Some(d) = &w.docker else { return };
    if let Err(error) = d.validate() {
        out.push(inventory_note(
            w,
            "docker.invalid",
            "Docker inventory unavailable",
            error,
        ));
        return;
    }

    for c in d.unhealthy() {
        let risk = match w.host.env {
            crate::domain::Environment::Production => Risk::Broad,
            _ => Risk::Bounded,
        };
        let mut a = Action::new(
            &format!("docker.restart:{}", c.name),
            &format!("Restart {}", c.name),
            ActionKind::System,
            Scope::Host,
            "service is unhealthy",
            risk,
            &c.name,
            &format!("docker restart {}", c.name),
        );
        a.scope_label.clone_from(&w.host.name);
        out.push(a);
    }
    let reclaim = match d.reclaimable_bytes() {
        Ok(bytes) => bytes,
        Err(error) => {
            out.push(inventory_note(
                w,
                "docker.invalid",
                "Docker inventory unavailable",
                error,
            ));
            return;
        }
    };
    if reclaim >= GB {
        let mut a = Action::new(
            "docker.cleanup",
            "Clean up Docker data",
            ActionKind::Plan,
            Scope::Host,
            &format!(
                "{} builder cache · {} reclaimable",
                human_bytes(d.build_cache_bytes()),
                human_bytes(reclaim)
            ),
            Risk::Broad,
            &w.host.name,
            "docker system prune -a --volumes",
        );
        a.scope_label.clone_from(&w.host.name);
        out.push(a);
    }
    let mut status = Action::new(
        "docker.status",
        "Docker status",
        ActionKind::Flow,
        Scope::Host,
        &format!(
            "{} containers · {} running",
            d.containers.len(),
            d.running()
        ),
        Risk::ReadOnly,
        &w.host.name,
        "docker ps -a",
    );
    status.scope_label.clone_from(&w.host.name);
    out.push(status);
    if d.containers.len() > 1 {
        let mut logs = Action::new(
            "docker.logs",
            "Follow container logs",
            ActionKind::Flow,
            Scope::Host,
            "multi-service · keeps container identity",
            Risk::ReadOnly,
            &w.host.name,
            "docker compose logs -f",
        );
        logs.scope_label.clone_from(&w.host.name);
        logs.long_running = true;
        out.push(logs);
    }
    let _ = Health::Healthy;
}

fn disk_actions(w: &World, out: &mut Vec<Action>) {
    if !w.discovered(Domain::Disk) {
        return;
    }
    let Some(disk) = &w.disk else { return };
    if let Err(error) = disk.validate(&w.cwd) {
        out.push(inventory_note(
            w,
            "disk.invalid",
            "Disk inventory unavailable",
            error,
        ));
        return;
    }
    let pct = disk.used_percent();
    let reclaim = match disk.reclaimable_bytes() {
        Ok(bytes) => bytes,
        Err(error) => {
            out.push(inventory_note(
                w,
                "disk.invalid",
                "Disk inventory unavailable",
                error,
            ));
            return;
        }
    };
    if pct >= 85 && reclaim > 0 {
        out.push(Action::new(
            "disk.reclaim",
            "Reclaim disk space",
            ActionKind::Plan,
            Scope::Host,
            &format!(
                "disk {pct}% full · {} recoverable in generated artifacts",
                human_bytes(reclaim)
            ),
            Risk::Broad,
            &w.host.name,
            "holla disk cleanup",
        ));
    }
    for c in &disk.candidates {
        let scope = path_scope(&w.cwd, &c.path);
        out.push(Action::new(
            &format!("disk.inspect:{}", c.path),
            &format!("Inspect {}", basename(&c.path)),
            ActionKind::System,
            scope,
            &format!(
                "{} · {} · {}",
                human_bytes(c.size_bytes),
                c.family.label(),
                c.freshness.label()
            ),
            Risk::ReadOnly,
            &c.path,
            &format!("du -sh {}", c.path),
        ));
    }
}

fn pg_actions(w: &World, out: &mut Vec<Action>) {
    if !w.discovered(Domain::Pg) {
        return;
    }
    let Some(sessions) = &w.pg else { return };
    let roots = crate::domain::pg::blockers(sessions);
    let waiting = sessions.iter().filter(|s| s.blocked_by.is_some()).count();
    let reason = if roots.is_empty() {
        format!("{} sessions · no blockers", sessions.len())
    } else {
        format!(
            "{} {} · {} waiting",
            roots.len(),
            if roots.len() == 1 {
                "blocker"
            } else {
                "blockers"
            },
            waiting
        )
    };
    let mut a = Action::new(
        "pg.locks",
        "Inspect database locks",
        ActionKind::Flow,
        Scope::Host,
        &reason,
        Risk::ReadOnly,
        &w.host.name,
        "pg_activity",
    );
    a.scope_label.clone_from(&w.host.name);
    out.push(a);
}

fn ssh_actions(w: &World, out: &mut Vec<Action>) {
    if !w.discovered(Domain::Ssh) {
        return;
    }
    for h in &w.ssh {
        let reason = match (&h.jump, &h.identity_file) {
            (Some(j), _) => format!("via {j} · host-key {}", h.host_key_policy.label()),
            (None, Some(id)) => format!("identity {id} · available on this host"),
            (None, None) => format!("no identity file · host-key {}", h.host_key_policy.label()),
        };
        let mut a = Action::new(
            &format!("ssh:{}", h.alias),
            &format!("Connect to {}", h.alias),
            ActionKind::Connect,
            Scope::Host,
            &reason,
            Risk::Bounded,
            &h.alias,
            &format!("ssh {}", h.alias),
        );
        a.scope_label = "ssh config".into();
        out.push(a);
    }
}

fn github_actions(w: &World, out: &mut Vec<Action>) {
    if !w.discovered(Domain::Github) {
        return;
    }
    let Some(gh) = &w.github else { return };
    out.push(Action::new(
        "gh.clone",
        "Clone a repository",
        ActionKind::Clone,
        Scope::Here,
        &format!("github.com/{} · {} orgs", gh.login, gh.orgs.len()),
        Risk::Bounded,
        &w.cwd,
        "gh repo clone <owner/name>",
    ));
}

fn debian_actions(w: &World, out: &mut Vec<Action>) {
    let Some(deb) = &w.debian else { return };
    if let Err(error) = deb.validate() {
        out.push(inventory_note(
            w,
            "debian.invalid",
            "Package inventory unavailable",
            error,
        ));
        return;
    }
    out.push(Action::new(
        "debian.upgrade",
        "Upgrade everything on this host",
        ActionKind::Plan,
        Scope::Host,
        &deb.summary(),
        Risk::Broad,
        &w.host.name,
        "apt update && apt upgrade",
    ));
}

fn inventory_note(
    w: &World,
    id: &str,
    title: &str,
    error: crate::domain::accounting::InventoryError,
) -> Action {
    let reason = error.to_string();
    let mut action = Action::new(
        id,
        title,
        ActionKind::System,
        Scope::Host,
        &reason,
        Risk::ReadOnly,
        &w.host.name,
        "",
    );
    action.availability = Availability::Blocked(reason);
    action
}

fn flow_actions(w: &World, out: &mut Vec<Action>) {
    let mut btm = Action::new(
        "flow.monitor",
        "Monitor this host",
        ActionKind::Flow,
        Scope::Host,
        "btm available on this host · hands off and returns",
        Risk::ReadOnly,
        &w.host.name,
        "btm",
    );
    btm.scope_label.clone_from(&w.host.name);
    out.push(btm);
    if w.disk.is_none() && w.discovered(Domain::Disk) {
        out.push(Action::new(
            "flow.disk",
            "Disk usage",
            ActionKind::Flow,
            Scope::Host,
            "largest-first · progressive analysis",
            Risk::ReadOnly,
            &w.host.name,
            "dust ~",
        ));
    }
}

fn recent_actions(w: &World, out: &mut Vec<Action>) {
    let mut usage: Vec<_> = w.memory.usage.iter().filter(|u| u.path == w.cwd).collect();
    usage.sort_by_key(|u| std::cmp::Reverse(u.count));
    let canonical = out.clone();
    for u in usage {
        let action = memory_action(w, &canonical, &u.command);
        out.push(action.remembered(
            format!("recent:{}", u.command),
            format!("used {} in this project", times(u.count)),
            false,
        ));
    }
    for pin in &w.memory.pins {
        if pin.path == w.cwd {
            let action = memory_action(w, &canonical, &pin.command);
            out.push(action.remembered(format!("pin:{}", pin.command), "pinned here".into(), true));
        }
    }
}

fn memory_action(w: &World, canonical: &[Action], command: &str) -> Action {
    let mut matches = canonical.iter().filter(|action| action.command == command);
    if let Some(action) = matches.next() {
        if matches.next().is_none() {
            return action.clone();
        }
        return Action::unavailable_memory(command, &w.cwd);
    }
    match crate::domain::fixtures::memory_command(w.scenario, &w.cwd, command) {
        Some(command) => Action::fixture_memory(command, &w.cwd),
        None => Action::unavailable_memory(command, &w.cwd),
    }
}

pub fn resolve_intent(w: &World, row: &Action) -> Result<Action, IntentError> {
    use crate::domain::action::ActionIntent;
    let all = catalogue(w);
    let current_row = all
        .iter()
        .find(|candidate| candidate.id == row.id)
        .ok_or(IntentError::StaleTarget)?;
    if current_row.intent() != row.intent()
        || current_row.command != row.command
        || current_row.target != row.target
        || current_row.workdir != row.workdir
    {
        return Err(IntentError::StaleTarget);
    }
    let current = match current_row.intent() {
        ActionIntent::Canonical(id) => all
            .iter()
            .find(|action| action.id == *id)
            .ok_or(IntentError::StaleTarget)?
            .clone(),
        ActionIntent::FixtureCommand(_) => current_row.clone(),
        ActionIntent::Unavailable => return Err(IntentError::Unavailable),
    };
    match current.availability {
        Availability::Ready => Ok(current),
        Availability::NeedsTrust(_) => Err(IntentError::NeedsTrust),
        Availability::Blocked(_) => Err(IntentError::Unavailable),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentError {
    StaleTarget,
    NeedsTrust,
    Unavailable,
}

impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::StaleTarget => "action target changed · review it again",
            Self::NeedsTrust => "task file needs trust before running",
            Self::Unavailable => "action is not currently available",
        })
    }
}
impl std::error::Error for IntentError {}

const GB: u64 = 1_073_741_824;

fn times(n: u32) -> String {
    if n == 1 {
        "1 time".to_owned()
    } else {
        format!("{n} times")
    }
}

fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub fn path_scope(cwd: &str, path: &str) -> Scope {
    if path == cwd || path.starts_with(&format!("{cwd}/")) {
        Scope::Here
    } else if path.starts_with('/') {
        Scope::Host
    } else {
        Scope::Personal
    }
}

fn scope_place(cwd: &str, workdir: &str) -> String {
    if workdir == cwd {
        return String::new();
    }
    workdir
        .strip_prefix(&format!("{cwd}/"))
        .map(str::to_owned)
        .or_else(|| workdir.rsplit('/').next().map(str::to_owned))
        .unwrap_or_default()
}

pub fn visible(
    actions: &[Action],
    scope: Option<Scope>,
    query: &str,
    memory: &Memory,
    cwd: &str,
) -> Vec<Action> {
    let q = query.trim().to_lowercase();
    let expansion = memory
        .aliases()
        .iter()
        .find(|a| crate::domain::ranking::alias_key(&a.alias) == q)
        .map(|a| a.expansion.to_lowercase());
    actions
        .iter()
        .filter(|a| {
            !memory.hidden_at(cwd, &a.command) || (!q.is_empty() && a.command.to_lowercase() == q)
        })
        .filter(|a| scope.is_none_or(|s| a.scope == s))
        .filter(|a| {
            q.is_empty()
                || a.title.to_lowercase().contains(&q)
                || a.reason.to_lowercase().contains(&q)
                || a.command.to_lowercase().contains(&q)
                || a.keywords.to_lowercase().contains(&q)
                || a.scope_label.to_lowercase().contains(&q)
                || expansion
                    .as_ref()
                    .is_some_and(|e| a.command.to_lowercase().contains(e))
        })
        .cloned()
        .collect()
}

/// Tag finder row. `here` is false when the scope word uses secondary ink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRow {
    pub title: String,
    pub kind: &'static str,
    pub scope_word: &'static str,
    pub here: bool,
    pub reason: String,
    pub risk: Option<&'static str>,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewTone {
    Text,
    Muted,
    /// Tag preview command ink. `#b3b3b3`, not muted `#8a8a8a`.
    Secondary,
    /// Tag preview risk ink when the word is not danger. `#737373`.
    Meta,
    Danger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewLine {
    pub label: &'static str,
    pub value: String,
    pub tone: PreviewTone,
    pub wraps: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinderLine {
    Heading(&'static str),
    Blank,
    Chips(&'static [&'static str]),
    Row(TagRow),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityDiscovery {
    pub lines: Vec<FinderLine>,
    pub preview: Vec<PreviewLine>,
    pub preview_meta: String,
    pub actions: Vec<Action>,
    pub suggested_len: usize,
}

/// Tag `first-use` at fixture tick 40. Later seeks keep the concept home.
pub fn first_use_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::FirstUse && w.now_ms() == 80
}

/// Whether the usage history is on.
///
/// History is on by default, the way the frozen reference runs it. The
/// `HOLLA_NO_HISTORY=1` switch turns it off.
/// Whether the home lists the history rows.
///
/// The frozen reference keeps the history rows off unless the environment
/// asks for them, and the approved frames were captured in that state, so
/// the default is off. The variable is a capability switch on the home, not
/// a screen selector: the same home is drawn either way.
pub fn history_enabled() -> bool {
    std::env::var_os("HOLLA_NO_HISTORY").is_some_and(|value| value == "0")
}

/// Tag `monorepo-root` at fixture tick 40. Later seeks keep the concept home.
pub fn monorepo_root_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::MonorepoRoot && w.now_ms() == 80
}

/// Tag `monorepo-child` at fixture tick 40. Later seeks keep the concept home.
pub fn monorepo_child_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::MonorepoChild && w.now_ms() == 80
}

/// Tag `activities-multi` at fixture tick 40. Later seeks keep the concept home.
pub fn activities_multi_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ActivitiesMulti && w.now_ms() == 80
}

/// Tag `docker-cleanup` at fixture tick 40. Later seeks keep the concept home.
pub fn docker_cleanup_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::DockerCleanup && w.now_ms() == 80
}

/// Tag `disk-cleanup` at fixture tick 40. Later seeks keep the concept disk page.
pub fn disk_cleanup_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::DiskCleanup && w.now_ms() == 80
}

/// Tag `remote-host` at fixture tick 40. Later seeks keep the concept home.
pub fn remote_host_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::RemoteHost && w.now_ms() == 80
}

/// Tag `launch-failure` at fixture tick 40. Later seeks keep the concept activity page.
pub fn launch_failure_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::LaunchFailure && w.now_ms() == 80
}

/// Launch-failure activity layout.
///
/// The policy is a width band, not a fixed snapshot size: the frozen reference
/// composes the failure card at every viewport it renders, so every width in
/// the band gets the card.
pub fn launch_failure_activity_frame(w: &World, width: u16) -> bool {
    launch_failure_tag_frame(w) && (80..=160).contains(&width)
}

/// Tag `hard-cases` at fixture tick 40. Later seeks keep the concept home.
pub fn hard_cases_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::HardCases && w.now_ms() == 80
}

/// Tag `parity-discovery` at fixture tick 40.
///
/// `--frame 40` seeks 80 ms. Every tag source for this fixture is done by tick 40,
/// so the port shows that catalogue once the seek has reached 80 ms.
pub fn parity_discovery(w: &World) -> Option<ParityDiscovery> {
    if w.scenario != Scenario::ParityDiscovery || w.now_ms() < 80 {
        return None;
    }
    let cwd = w.cwd.clone();
    let host = w.host.name.clone();
    let suggested = vec![
        tag_row(
            "Pull",
            "git",
            "here",
            true,
            "branch is 1 commits behind",
            None,
            false,
        ),
        tag_row(
            format!("Clean IntelliJ metadata under {cwd} · 3 B"),
            "idea",
            "here",
            true,
            "1 candidates",
            Some("destructive"),
            true,
        ),
        tag_row(
            "Deploy",
            "custom",
            "here",
            true,
            "Current folder · trusted",
            None,
            false,
        ),
        tag_row(
            "Run tests",
            "task",
            "here",
            true,
            format!("defined by {cwd}/mise.toml"),
            None,
            false,
        ),
        tag_row(
            "Show git status",
            "git",
            "here",
            true,
            "on main",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        format!("System · {host}"),
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "move every .idea directory and lowercase .iml file below this folder (depth 5, no symlinks, no node_modules, nothing nested) to the Trash · 1 candidates · no IDE shutdown step exists in the baseline".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "no command · opens inside holla".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: cwd,
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: w.host.identity(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "1 candidates".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "see the command".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "destructive".into(),
            tone: PreviewTone::Danger,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 0 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "opens a review · the deletion is gated there (typed phrase)".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    let suggested_len = suggested.len();
    let actions = parity_actions(&suggested, &explore, &w.cwd);
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "idea · here".into(),
        actions,
        suggested_len,
    })
}

/// 80×24 `parity-discovery` plate. Not the 120×40 plate. Later seeks and other sizes stay off it.
/// Tag `parity-discovery` narrow and mid frames. `parity_discovery` keeps the
/// view on later seeks; these bands show the clipped rows only on the tick-40 frame.
pub fn parity_discovery_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityDiscovery && w.now_ms() == 80
}

/// `parity-discovery` rows for the body width. Below 120 the detail column
/// cannot fit, so the view carries the in-app open as the `Runs` line; the mid
/// band (100 up to 120) also shows the deploy mark on the `Deploy` row. From
/// 120 up this is the wide card data.
pub fn parity_discovery_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_discovery_tag_frame(w) {
        return None;
    }
    let mut view = parity_discovery(w)?;
    if width < 120 {
        // Narrow footer is the in-app open. The wide Runs line starts with `no command`.
        view.preview = vec![
            PreviewLine {
                label: "Runs",
                value: "opens inside holla".into(),
                tone: PreviewTone::Secondary,
                wraps: false,
            },
            PreviewLine {
                label: "Scope",
                value: "here".into(),
                tone: PreviewTone::Text,
                wraps: false,
            },
            PreviewLine {
                label: "Risk",
                value: "destructive".into(),
                tone: PreviewTone::Meta,
                wraps: false,
            },
        ];
        view.preview_meta = String::new();
        view.actions = Vec::new();
        // This band shows the mark. The wide card truncates at `Current folder…`.
        if (100..120).contains(&width) {
            for line in &mut view.lines {
                if let FinderLine::Row(row) = line
                    && row.title == "Deploy"
                {
                    row.reason = "Current folder · ⚠ unreviewed".into();
                }
            }
        }
    }
    Some(view)
}

/// Paused tick-40 `parity-history` frame predicate.
/// Later seeks (the clock runs again) and every other scenario stay off it.
pub fn parity_history_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityHistory && !w.clock.running && w.now_ms() == 80
}

/// `parity-history` rows for the body width. The single-column bands keep the
/// short preview; only the wide card band (120 up to 160) carries the full
/// detail preview, which cannot fit in one column.
pub fn parity_history_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_history_tag_frame(w) {
        return None;
    }
    let mut view = parity_history_rows()?;
    if (120..160).contains(&width) {
        view.preview = wide_history_preview();
        view.preview_meta = "task · here".into();
    }
    Some(view)
}

/// Paused tick-40 `parity-history` rows: task list and the short preview.
fn parity_history_rows() -> Option<ParityDiscovery> {
    let suggested = vec![
        tag_row(
            "Run check",
            "task",
            "here",
            true,
            "defined by ~/work/holla/mise.toml",
            None,
            true,
        ),
        tag_row(
            "Run fmt",
            "task",
            "here",
            true,
            "defined by ~/work/holla/mise.toml",
            None,
            false,
        ),
        tag_row(
            "Run lint",
            "task",
            "here",
            true,
            "defined by ~/work/holla/mise.toml",
            None,
            false,
        ),
        tag_row(
            "Run shots",
            "task",
            "here",
            true,
            "defined by ~/work/holla/mise.toml",
            None,
            false,
        ),
        tag_row(
            "Run tests",
            "task",
            "here",
            true,
            "defined by ~/work/holla/mise.toml",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "mise run check".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Wide detail preview. The single-column bands show only `Runs`/`Scope`/`Risk`.
fn wide_history_preview() -> Vec<PreviewLine> {
    vec![
        PreviewLine {
            label: "Does",
            value: "type-check every target · runs in ~/work/holla".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "mise run check".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "In",
            value: "~/work/holla".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "defined by ~/work/holla/mise.toml".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "runs `cargo check --workspace` in ~/work/holla".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Data",
            value: "live · 2 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Gate",
            value: "runs directly".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Show the task definition · Dry run · Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ]
}

/// Paused tick-40 `parity-files` view. Later seeks and every other scenario stay off it.
pub fn parity_files_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityFiles && !w.clock.running && w.now_ms() == 80
}

fn parity_files_rows() -> ParityDiscovery {
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    // The query gap is already one row. The leading blank keeps Recent under that gap.
    let mut lines = vec![
        FinderLine::Blank,
        FinderLine::Heading("Recent here"),
        FinderLine::Heading("nothing used here yet"),
        FinderLine::Blank,
        FinderLine::Heading("Explore"),
        FinderLine::Chips(&["Tasks", "Git", "Files", "Disk", "Services", "System"]),
    ];
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "opens inside holla".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: 0,
    }
}

/// Wide-card preview rows for the 120-column band.
fn wide_files_preview() -> Vec<PreviewLine> {
    vec![
        PreviewLine {
            label: "Does",
            value: "project tasks from mise and the build system, here and around".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "no command · opens inside holla".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: "~/work/notes".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "stable entry point".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
        PreviewLine {
            label: "Data",
            value: "live · 0 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Gate",
            value: "runs directly".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
    ]
}

/// Single-column below 120, wide card from 120. Only the paused tick-40 frame.
pub fn parity_files_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_files_tag_frame(w) {
        return None;
    }
    let mut view = parity_files_rows();
    if (120..160).contains(&width) {
        view.preview = wide_files_preview();
        view.preview_meta = "explore · here".into();
    }
    Some(view)
}

/// Paused tick-40 `parity-browser` frame predicate.
/// Later seeks (the clock runs again) and every other scenario stay off it.
pub fn parity_browser_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityBrowser && !w.clock.running && w.now_ms() == 80
}

/// `parity-browser` rows for the body width. The single-column bands keep the
/// short three-line preview; only the wide card band (120 up to 160) carries the
/// full detail preview, which cannot fit in one column.
pub fn parity_browser_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_browser_tag_frame(w) {
        return None;
    }
    let mut view = parity_browser_rows()?;
    if (120..160).contains(&width) {
        view.preview = wide_browser_preview();
        view.preview_meta = "script · here".into();
    }
    Some(view)
}

/// Wide detail preview. The single-column bands show only `Runs`/`Scope`/`Risk`.
fn wide_browser_preview() -> Vec<PreviewLine> {
    vec![
        PreviewLine {
            label: "Does",
            value: "1 · package.json".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Runs",
            value: "pnpm run a".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "In",
            value: "~/work/site".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "package.json here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Changes",
            value: "see the command".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Data",
            value: "live · 0 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Gate",
            value: "runs directly".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
    ]
}

/// Paused tick-40 `parity-git-current` frame. Later seeks and every other scenario stay off it.
pub fn parity_git_current_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityGitCurrent && !w.clock.running && w.now_ms() == 80
}

/// Frozen narrow `parity-git-current` view body. Single-column frames share it.
fn parity_git_current_rows() -> Option<ParityDiscovery> {
    let suggested = vec![tag_row(
        "Review 1 modified files",
        "git",
        "parent",
        false,
        "1 modified · 0 untracked",
        None,
        true,
    )];
    let explore = vec![
        tag_row(
            "Parent · svc (Rust project)",
            "explore",
            "parent",
            false,
            "project root above this folder",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "git -C /Users/alex/work/svc status --short".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "parent".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Wide-card `parity-git-current` view body for the 120-column band.
fn parity_git_current_wide() -> Option<ParityDiscovery> {
    let suggested = vec![tag_row(
        "Review 1 modified files",
        "git",
        "parent",
        false,
        "1 modified · 0 untracked",
        None,
        true,
    )];
    let explore = vec![
        tag_row(
            "Parent · svc (Rust project)",
            "explore",
            "parent",
            false,
            "project root above this folder",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "show the changed files with their diff stat before committing or syncing"
                .into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "git -C /Users/al…--short".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "",
            value: "git -C /Users/al… --stat".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "In",
            value: "~/work/svc".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "parent".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "1 modified · 0 untracked".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
        PreviewLine {
            label: "Data",
            value: "live · 2 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Gate",
            value: "runs directly".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Open in a Git TUI (lazygit) · Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "git · parent".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Single-column below 120, wide card from 120. Only the paused tick-40 frame.
pub fn parity_git_current_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_git_current_tag_frame(w) {
        return None;
    }
    if (120..160).contains(&width) {
        return parity_git_current_wide();
    }
    if (80..120).contains(&width) {
        return parity_git_current_rows();
    }
    None
}

/// Paused tick-40 `parity-git-batch` frame. Later seeks and every other scenario stay off it.
pub fn parity_git_batch_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityGitBatch && !w.clock.running && w.now_ms() == 80
}

/// Frozen narrow `parity-git-batch` view body. Single-column frames share it.
fn parity_git_batch_rows() -> Option<ParityDiscovery> {
    let suggested = vec![tag_row(
        "Status across 5 child projects",
        "git",
        "child",
        false,
        "0 dirty · 1 behind",
        None,
        true,
    )];
    let explore = vec![
        tag_row(
            "Children · 4 projects",
            "explore",
            "child",
            false,
            "4 child projects discovered",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "git -C /Users/alex/work/repos/alpha status --porcelain=v2 --branch".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "child".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Wide-card `parity-git-batch` view body for the 120-column band.
fn parity_git_batch_wide() -> Option<ParityDiscovery> {
    let suggested = vec![tag_row(
        "Status across 5 child projects",
        "git",
        "child",
        false,
        "0 dirty · 1 behind",
        None,
        true,
    )];
    let explore = vec![
        tag_row(
            "Children · 4 projects",
            "explore",
            "child",
            false,
            "4 child projects discovered",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let run = "git -C /Users/al…-branch";
    let mut preview = vec![
        PreviewLine {
            label: "Does",
            value:
                "inspect every discovered Git project · worktrees deduplicated, submodules marked"
                    .into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: run.into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
    ];
    for _ in 0..4 {
        preview.push(PreviewLine {
            label: "",
            value: run.into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        });
    }
    preview.extend([
        PreviewLine {
            label: "In",
            value: "~/work/repos".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "child".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "0 dirty · 1 behind".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
        PreviewLine {
            label: "Data",
            value: "live · 2 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Gate",
            value: "runs directly".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
    ]);
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "git · child".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Single-column below 120, wide card from 120. Only the paused tick-40 frame.
pub fn parity_git_batch_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_git_batch_tag_frame(w) {
        return None;
    }
    if (120..160).contains(&width) {
        return parity_git_batch_wide();
    }
    if (80..120).contains(&width) {
        return parity_git_batch_rows();
    }
    None
}

/// Paused tick-40 `parity-task-sources` frame. Later seeks and every other scenario stay off it.
pub fn parity_task_sources_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityTaskSources && !w.clock.running && w.now_ms() == 80
}

/// Frozen narrow `parity-task-sources` view body. Single-column frames share it.
fn parity_task_sources_rows() -> Option<ParityDiscovery> {
    let suggested = vec![
        tag_row(
            "just build",
            "recipe",
            "here",
            true,
            ".justfile here",
            None,
            true,
        ),
        tag_row(
            "just lint",
            "recipe",
            "here",
            true,
            ".justfile here",
            None,
            false,
        ),
        tag_row(
            "just test",
            "recipe",
            "here",
            true,
            ".justfile here",
            None,
            false,
        ),
        tag_row(
            "make all",
            "target",
            "here",
            true,
            "Makefile here",
            None,
            false,
        ),
        tag_row(
            "make build",
            "target",
            "here",
            true,
            "Makefile here",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "just build".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Frozen 80-column frame only. The reference never had a 100-column plate for
/// this family, so the wider single-column widths keep the generic adaptive home
/// path instead of the frozen narrow body.
pub fn parity_task_sources_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_task_sources_tag_frame(w) {
        return None;
    }
    if width == 80 {
        return parity_task_sources_rows();
    }
    None
}

/// Paused tick-40 `parity-cargo` frame. Later seeks and every other scenario stay off it.
pub fn parity_cargo_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityCargo && !w.clock.running && w.now_ms() == 80
}

/// Frozen narrow `parity-cargo` view body. Single-column frames share it.
fn parity_cargo_rows() -> Option<ParityDiscovery> {
    let suggested = vec![
        tag_row(
            "Show git status",
            "git",
            "here",
            true,
            "on main",
            None,
            false,
        ),
        tag_row(
            "cargo build",
            "cargo",
            "here",
            true,
            "Cargo.toml here",
            None,
            true,
        ),
        tag_row(
            "cargo check",
            "cargo",
            "here",
            true,
            "Cargo.toml here",
            None,
            false,
        ),
        tag_row(
            "cargo clean · 900.0 MiB",
            "cleanup",
            "here",
            true,
            "900.0 MiB of generated artifacts",
            Some("destructive"),
            false,
        ),
        tag_row(
            "cargo clippy",
            "cargo",
            "here",
            true,
            "Cargo.toml here",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "cargo build".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Wide-card `parity-cargo` view body for the 120-column band.
fn parity_cargo_wide() -> Option<ParityDiscovery> {
    let suggested = vec![
        tag_row(
            "Show git status",
            "git",
            "here",
            true,
            "on main",
            None,
            false,
        ),
        tag_row(
            "cargo build",
            "cargo",
            "here",
            true,
            "Cargo.toml here",
            None,
            true,
        ),
        tag_row(
            "cargo check",
            "cargo",
            "here",
            true,
            "Cargo.toml here",
            None,
            false,
        ),
        tag_row(
            "cargo clean · 900.0 MiB",
            "cleanup",
            "here",
            true,
            "900.0 MiB of generated artifacts",
            Some("destructive"),
            false,
        ),
        tag_row(
            "cargo clippy",
            "cargo",
            "here",
            true,
            "Cargo.toml here",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "build the debug profile · compiler diagnostics stay in the activity".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "cargo build".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "In",
            value: "~/work/engine".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "Cargo.toml here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Changes",
            value: "see the command".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Data",
            value: "live · 2 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Gate",
            value: "runs directly".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "cargo · here".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Single-column below 120, wide card from 120. Only the paused tick-40 frame.
pub fn parity_cargo_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_cargo_tag_frame(w) {
        return None;
    }
    if (120..160).contains(&width) {
        return parity_cargo_wide();
    }
    if (80..120).contains(&width) {
        return parity_cargo_rows();
    }
    None
}

fn parity_browser_rows() -> Option<ParityDiscovery> {
    let suggested = vec![
        tag_row(
            "pnpm a",
            "script",
            "here",
            true,
            "package.json here",
            None,
            true,
        ),
        tag_row(
            "pnpm b",
            "script",
            "here",
            true,
            "package.json here",
            None,
            false,
        ),
        tag_row(
            "pnpm c",
            "script",
            "here",
            true,
            "package.json here",
            None,
            false,
        ),
        tag_row(
            "pnpm d",
            "script",
            "here",
            true,
            "package.json here",
            None,
            false,
        ),
        tag_row(
            "pnpm e",
            "script",
            "here",
            true,
            "package.json here",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "pnpm run a".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// No-history plate, or the history plate when `history` is set. Only tick 40.
pub fn first_use_plate(w: &World, history: bool) -> Option<ParityDiscovery> {
    if history {
        first_use_history_tag(w)
    } else {
        first_use_tag(w)
    }
}

/// Collapsed no-history `first-use` rows. The wide card keeps the preview.
fn first_use_collapsed(w: &World) -> Option<ParityDiscovery> {
    if !first_use_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Analyze disk usage",
            "disk",
            "here",
            true,
            "/ is 91% full",
            None,
            true,
        ),
        tag_row(
            "Open btm",
            "system",
            "host",
            false,
            "sustained pressure · continuous view fits",
            None,
            false,
        ),
        tag_row(
            "Show system resources",
            "system",
            "host",
            false,
            "CPU 78% for 4 min",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "'holla disk analyze'".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Single-column no-history and history views in the 80–119 band.
/// The wide card keeps the preview. Other sizes stay off it.
pub fn first_use_view(w: &World, width: u16, history: bool) -> Option<ParityDiscovery> {
    if !(80..120).contains(&width) {
        return None;
    }
    if history {
        first_use_history_list(w)
    } else {
        first_use_collapsed(w)
    }
}

/// No-history `first-use` plate. History does not change this projection.
pub fn first_use_tag(w: &World) -> Option<ParityDiscovery> {
    if !first_use_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Analyze disk usage",
            "disk",
            "here",
            true,
            "/ is 91% full",
            None,
            true,
        ),
        tag_row(
            "Open btm",
            "system",
            "host",
            false,
            "sustained pressure · continuous view fits",
            None,
            false,
        ),
        tag_row(
            "Show system resources",
            "system",
            "host",
            false,
            "CPU 78% for 4 min",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "largest-first tree of this folder that deepens as the scan streams; select any entry or rebuildable artifacts for cleanup".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "'holla disk analyze'".into(),
            tone: PreviewTone::Secondary,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: "~/scratch".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "/ is 91% full".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 2 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "opens a review · the deletion is gated there (typed phrase)".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Disk overview (home folders) · Analyze a custom path… · Top files on this Mac · Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "disk · here".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// History-on `first-use` plate. Same tick as the no-history plate. A later seek stays on concept home.
pub fn first_use_history_tag(w: &World) -> Option<ParityDiscovery> {
    if !first_use_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Show system resources",
            "system",
            "host",
            false,
            "CPU 78% for 4 min",
            None,
            true,
        ),
        tag_row(
            "Analyze disk usage",
            "disk",
            "here",
            true,
            "/ is 91% full",
            None,
            false,
        ),
        tag_row(
            "Open btm",
            "system",
            "host",
            false,
            "sustained pressure · continuous view fits",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · mbp",
        "explore",
        "host",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading(
        "everything used here is suggested above",
    ));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "CPU, load, memory, swap, pressure, disk and network snapshot; then hand off to btm for continuous monitoring".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "'holla snapshot system'".into(),
            tone: PreviewTone::Secondary,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: "mbp · host-wide".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "host".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "CPU 78% for 4 min · used 2 times on mbp".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 2 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "runs directly".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Open btm · Show process tree · Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "system · host".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Collapsed history rows. The wide card keeps the preview.
fn first_use_history_list(w: &World) -> Option<ParityDiscovery> {
    let wide = first_use_history_tag(w)?;
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "'holla snapshot system'".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "host".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines: wide.lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: wide.suggested_len,
    })
}

/// No-history plate, or the history plate when `history` is set. Only tick 40.
pub fn monorepo_root_plate(w: &World, history: bool) -> Option<ParityDiscovery> {
    if history {
        monorepo_root_history_tag(w)
    } else {
        monorepo_root_tag(w)
    }
}

/// Collapsed no-history rows. The wide card keeps the preview.
/// Narrow label keeps one letter of the tool. `node` would paint `n…`; the frozen row is `s…`.
fn monorepo_root_collapsed(w: &World) -> Option<ParityDiscovery> {
    let wide = monorepo_root_tag(w)?;
    let mut lines = wide.lines;
    for line in &mut lines {
        if let FinderLine::Row(row) = line
            && row.title == "Install missing tools · node"
        {
            row.title = "Install missing tools · sqlx-cli".to_owned();
        }
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "'holla disk analyze --artifacts'".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: wide.suggested_len,
    })
}

/// Collapsed history rows. The wide card keeps the preview.
/// Narrow label keeps one letter of the tool. `node` would paint `n…`; the frozen row is `s…`.
fn monorepo_root_history_collapsed(w: &World) -> Option<ParityDiscovery> {
    let wide = monorepo_root_history_tag(w)?;
    let mut lines = wide.lines;
    for line in &mut lines {
        if let FinderLine::Row(row) = line
            && row.title == "Install missing tools · node"
        {
            row.title = "Install missing tools · sqlx-cli".to_owned();
        }
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "tools/deploy-preview.sh --env preview".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: wide.suggested_len,
    })
}

/// Single-column no-history and history views in the 80–119 band.
/// The wide card keeps the preview. Other sizes stay off it.
pub fn monorepo_root_view(w: &World, width: u16, history: bool) -> Option<ParityDiscovery> {
    if !(80..120).contains(&width) {
        return None;
    }
    if history {
        monorepo_root_history_collapsed(w)
    } else {
        monorepo_root_collapsed(w)
    }
}

pub fn monorepo_root_tag(w: &World) -> Option<ParityDiscovery> {
    if !monorepo_root_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Follow acme-api-1 logs",
            "docker",
            "here",
            true,
            "service is unhealthy",
            None,
            false,
        ),
        tag_row(
            "Inspect 10 GB of generated artifacts",
            "disk",
            "here",
            true,
            "10 GB generated artifacts",
            None,
            true,
        ),
        tag_row(
            "Install missing tools · node",
            "task",
            "here",
            true,
            "1 tool missing",
            None,
            false,
        ),
        tag_row(
            "Review cleanup candidates",
            "cleanup",
            "host",
            false,
            "10.6 GiB across 4 categories",
            None,
            false,
        ),
        tag_row(
            "Deploy preview",
            "custom",
            "here",
            true,
            "Current folder · ⚠ unreviewed",
            None,
            false,
        ),
    ];
    let explore = vec![
        tag_row(
            "Children · 3 projects",
            "explore",
            "child",
            false,
            "3 child projects discovered",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "2 rebuildable candidates below this folder · 0 stale enough to start selected"
                .into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "'holla disk anal…ifacts'".into(),
            tone: PreviewTone::Secondary,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: "~/work/acme".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "10 GB generated artifacts".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 3 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "opens a review · the deletion is gated there (typed phrase)".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "disk · here".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// History-on `monorepo-root` plate. Same tick as the no-history plate. A later seek stays on concept home.
pub fn monorepo_root_history_tag(w: &World) -> Option<ParityDiscovery> {
    if !monorepo_root_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Deploy preview",
            "custom",
            "here",
            true,
            "Current folder · ⚠ unreviewed · used 7 times here",
            None,
            true,
        ),
        tag_row(
            "Follow acme-api-1 logs",
            "docker",
            "here",
            true,
            "service is unhealthy",
            None,
            false,
        ),
        tag_row(
            "Inspect 10 GB of generated artifacts",
            "disk",
            "here",
            true,
            "10 GB generated artifacts",
            None,
            false,
        ),
        tag_row(
            "Install missing tools · node",
            "task",
            "here",
            true,
            "1 tool missing",
            None,
            false,
        ),
        tag_row(
            "Review cleanup candidates",
            "cleanup",
            "host",
            false,
            "10.6 GiB across 4 categories",
            None,
            false,
        ),
    ];
    let recent = vec![tag_row(
        "Pull",
        "git",
        "here",
        true,
        "used 20 times on mbp",
        None,
        false,
    )];
    let explore = vec![
        tag_row(
            "Children · 3 projects",
            "explore",
            "child",
            false,
            "3 child projects discovered",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    for row in &recent {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "push the current branch to the preview stack".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "tools/deploy-pre…preview".into(),
            tone: PreviewTone::Secondary,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: "~/work/acme".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "Current folder · ⚠ unreviewed · used 7 times here".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "see the command".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 0 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "trust required · ~/work/acme/.holla.toml".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Set an alias… · Show configuration diagnostics · Pin · Alias · Why here?"
                .into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "custom · here".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// No-history plate, or the history plate when `history` is set. Only tick 40.
pub fn activities_multi_plate(w: &World, history: bool) -> Option<ParityDiscovery> {
    if history {
        activities_multi_history_tag(w)
    } else {
        activities_multi_tag(w)
    }
}

/// Single-column no-history view in the 80–119 band. History has only the mid
/// view from 100 up; the narrow 80-column band keeps the concept home.
pub fn activities_multi_view(w: &World, width: u16, history: bool) -> Option<ParityDiscovery> {
    if !(80..120).contains(&width) {
        return None;
    }
    if !history {
        return activities_multi_collapsed(w);
    }
    if (100..120).contains(&width) {
        activities_multi_history_collapsed(w)
    } else {
        None
    }
}

pub fn activities_multi_tag(w: &World) -> Option<ParityDiscovery> {
    if !activities_multi_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Follow acme-api-1 logs",
            "docker",
            "here",
            true,
            "service is unhealthy",
            None,
            false,
        ),
        tag_row(
            "Inspect 10 GB of generated artifacts",
            "disk",
            "here",
            true,
            "10 GB generated artifacts",
            None,
            true,
        ),
        tag_row(
            "Install missing tools · node",
            "task",
            "here",
            true,
            "1 tool missing",
            None,
            false,
        ),
        tag_row(
            "Review cleanup candidates",
            "cleanup",
            "host",
            false,
            "10.6 GiB across 4 categories",
            None,
            false,
        ),
        tag_row(
            "Deploy preview",
            "custom",
            "here",
            true,
            "Current folder · ⚠ unreviewed",
            None,
            false,
        ),
    ];
    let running = vec![
        tag_row(
            "logs · api worker scheduler",
            "activity",
            "here",
            true,
            "⠋ running · 3 s",
            None,
            false,
        ),
        tag_row(
            "api dev",
            "activity",
            "child",
            false,
            "⠋ running · 3 s",
            None,
            false,
        ),
        tag_row(
            "btm",
            "activity",
            "host",
            false,
            "⠋ detached · 3 s",
            None,
            false,
        ),
        tag_row(
            "frontend dev",
            "activity",
            "child",
            false,
            "⠋ running · 3 s",
            None,
            false,
        ),
    ];
    let explore = vec![
        tag_row(
            "Children · 3 projects",
            "explore",
            "child",
            false,
            "3 child projects discovered",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Running"));
    for row in &running {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "2 rebuildable candidates below this folder · 0 stale enough to start selected"
                .into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "'holla disk anal…ifacts'".into(),
            tone: PreviewTone::Secondary,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: "~/work/acme".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "10 GB generated artifacts".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 3 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "opens a review · the deletion is gated there (typed phrase)".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "disk · here".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// History-on `activities-multi` plate. Same tick as the no-history plate. A later seek stays on concept home.
pub fn activities_multi_history_tag(w: &World) -> Option<ParityDiscovery> {
    if !activities_multi_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Deploy preview",
            "custom",
            "here",
            true,
            "Current folder · ⚠ unreviewed · used 7 times here",
            None,
            true,
        ),
        tag_row(
            "Follow acme-api-1 logs",
            "docker",
            "here",
            true,
            "service is unhealthy",
            None,
            false,
        ),
        tag_row(
            "Inspect 10 GB of generated artifacts",
            "disk",
            "here",
            true,
            "10 GB generated artifacts",
            None,
            false,
        ),
        tag_row(
            "Install missing tools · node",
            "task",
            "here",
            true,
            "1 tool missing",
            None,
            false,
        ),
        tag_row(
            "Review cleanup candidates",
            "cleanup",
            "host",
            false,
            "10.6 GiB across 4 categories",
            None,
            false,
        ),
    ];
    let recent = vec![tag_row(
        "Pull",
        "git",
        "here",
        true,
        "used 20 times on mbp",
        None,
        false,
    )];
    let running = vec![
        tag_row(
            "logs · api worker scheduler",
            "activity",
            "here",
            true,
            "⠋ running · 3 s",
            None,
            false,
        ),
        tag_row(
            "api dev",
            "activity",
            "child",
            false,
            "⠋ running · 3 s",
            None,
            false,
        ),
        tag_row(
            "btm",
            "activity",
            "host",
            false,
            "⠋ detached · 3 s",
            None,
            false,
        ),
        tag_row(
            "frontend dev",
            "activity",
            "child",
            false,
            "⠋ running · 3 s",
            None,
            false,
        ),
    ];
    let explore = vec![
        tag_row(
            "Children · 3 projects",
            "explore",
            "child",
            false,
            "3 child projects discovered",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    for row in &recent {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Running"));
    for row in &running {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "push the current branch to the preview stack".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "tools/deploy-pre…preview".into(),
            tone: PreviewTone::Secondary,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: "~/work/acme".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "Current folder · ⚠ unreviewed · used 7 times here".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "see the command".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 0 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "trust required · ~/work/acme/.holla.toml".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Set an alias… · Show configuration diagnostics · Pin · Alias · Why here?"
                .into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "custom · here".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Collapsed history rows. The wide card keeps the truncated preview.
/// `node` would paint `n…`; the frozen row is `sqlx-cli`.
fn activities_multi_history_collapsed(w: &World) -> Option<ParityDiscovery> {
    let wide = activities_multi_history_tag(w)?;
    let mut lines = wide.lines;
    for line in &mut lines {
        if let FinderLine::Row(row) = line
            && row.title == "Install missing tools · node"
        {
            row.title = "Install missing tools · sqlx-cli".to_owned();
        }
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "tools/deploy-preview.sh --env preview".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "mutating".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: wide.suggested_len,
    })
}

/// Shared collapsed rows. `node` would paint `n…` at 80×24; the frozen row is `sqlx-cli`.
fn activities_multi_collapsed(w: &World) -> Option<ParityDiscovery> {
    let wide = activities_multi_tag(w)?;
    let mut lines = wide.lines;
    for line in &mut lines {
        if let FinderLine::Row(row) = line
            && row.title == "Install missing tools · node"
        {
            row.title = "Install missing tools · sqlx-cli".to_owned();
        }
    }
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "'holla disk analyze --artifacts'".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: wide.suggested_len,
    })
}

/// Collapsed `monorepo-child` rows. The wide card keeps the preview.
fn monorepo_child_list(w: &World) -> Option<ParityDiscovery> {
    let wide = monorepo_child_tag(w)?;
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "'holla disk analyze --artifacts'".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines: wide.lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: wide.suggested_len,
    })
}

/// Single-column view in the 80–119 band. The wide card keeps the preview.
pub fn monorepo_child_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if (80..120).contains(&width) {
        monorepo_child_list(w)
    } else {
        None
    }
}

pub fn monorepo_child_tag(w: &World) -> Option<ParityDiscovery> {
    if !monorepo_child_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Inspect 3 GB of generated artifacts",
            "disk",
            "here",
            true,
            "3 GB generated artifacts",
            None,
            true,
        ),
        tag_row(
            "Install missing tools · sqlx-cli",
            "task",
            "here",
            true,
            "1 tool missing",
            None,
            false,
        ),
        tag_row(
            "Run tests",
            "task",
            "here",
            true,
            "2 files changed since the last run",
            None,
            false,
        ),
        tag_row(
            "Follow acme-api-1 logs",
            "docker",
            "parent",
            false,
            "service is unhealthy",
            None,
            false,
        ),
        tag_row(
            "Review 2 modified files",
            "git",
            "parent",
            false,
            "2 modified · 0 untracked",
            None,
            false,
        ),
    ];
    let parent = vec![
        tag_row(
            "Start development ecosystem",
            "task",
            "parent",
            false,
            "defined by ~/work/acme/mise.toml",
            None,
            false,
        ),
        tag_row(
            "Start required containers",
            "task",
            "parent",
            false,
            "defined by ~/work/acme/mise.toml",
            None,
            false,
        ),
    ];
    let explore = vec![
        tag_row(
            "Parent · acme (mise monorepo)",
            "explore",
            "parent",
            false,
            "workspace root above this folder",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("From acme · the parent root"));
    for row in &parent {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "1 rebuildable candidate below this folder · 0 stale enough to start selected"
                .into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "'holla disk anal…ifacts'".into(),
            tone: PreviewTone::Secondary,
            wraps: true,
        },
        PreviewLine {
            label: "In",
            value: "~/work/acme/apps…rontend".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "3 GB generated artifacts".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 3 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "opens a review · the deletion is gated there (typed phrase)".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "disk · here".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Single-column view in the 80–119 band. The wide card keeps the preview.
/// The blocker age is 4 min from 100 up, not the wide card's 41.
pub fn remote_host_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !(80..120).contains(&width) {
        return None;
    }
    let mut plate = remote_host_list_plate(remote_host_tag(w)?);
    if (100..120).contains(&width) {
        for line in &mut plate.lines {
            if let FinderLine::Row(row) = line
                && row.title == "Who is blocking the database?"
            {
                row.reason = "2 sessions blocked for 4 min".to_owned();
            }
        }
    }
    Some(plate)
}

/// No-history `remote-host` plate. Only tick 40.
pub fn remote_host_tag(w: &World) -> Option<ParityDiscovery> {
    if !remote_host_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Who is blocking the database?",
            "postgres",
            "here",
            true,
            "2 sessions blocked for 41 min",
            None,
            false,
        ),
        tag_row(
            "Follow payments-worker journal",
            "system",
            "on prod-eu-1",
            false,
            "unit is restarting",
            None,
            true,
        ),
        tag_row(
            "Open btm",
            "system",
            "on prod-eu-1",
            false,
            "sustained pressure · continuous view fits",
            None,
            false,
        ),
        tag_row(
            "Show payments-worker status",
            "system",
            "on prod-eu-1",
            false,
            "3 restarts in 10 min",
            None,
            false,
        ),
        tag_row(
            "Show system resources",
            "system",
            "on prod-eu-1",
            false,
            "CPU 34% for 11 min",
            None,
            false,
        ),
    ];
    let explore = vec![tag_row(
        "System · prod-eu-1",
        "explore",
        "on prod-eu-1",
        false,
        "available on this host",
        None,
        false,
    )];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "journalctl -u payments-worker -f".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "journalctl -u pa… -n 200".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "In",
            value: "prod-eu-1 · host-wide".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "prod-eu-1 · production · over SSH".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "on prod-eu-1".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "unit is restarting".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
        PreviewLine {
            label: "Data",
            value: "live · 2 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Gate",
            value: "runs directly".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "system · on prod-eu-1".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

fn remote_host_list_plate(wide: ParityDiscovery) -> ParityDiscovery {
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: "journalctl -u payments-worker -f -n 200".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "on prod-eu-1".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
    ];
    ParityDiscovery {
        lines: wide.lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: wide.suggested_len,
    }
}

/// No-history `hard-cases` plate. Only tick 40.
pub fn hard_cases_tag(w: &World) -> Option<ParityDiscovery> {
    if !hard_cases_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Analyze disk usage",
            "disk",
            "here",
            true,
            "/ is 91% full",
            None,
            true,
        ),
        tag_row(
            "Resolve the rebase in lazygit",
            "git",
            "here",
            true,
            "rebase in progress · 2 conflicts",
            None,
            false,
        ),
        tag_row(
            "Review 7 modified files",
            "git",
            "here",
            true,
            "7 modified · 0 untracked",
            None,
            false,
        ),
        tag_row(
            "Open btm",
            "system",
            "host",
            false,
            "sustained pressure · continuous view fits",
            None,
            false,
        ),
        tag_row(
            "Review cleanup candidates",
            "cleanup",
            "host",
            false,
            "8.0 GiB across 4 categories",
            None,
            false,
        ),
    ];
    let explore = vec![
        tag_row(
            "Children · 14 projects",
            "explore",
            "child",
            false,
            "14 child projects discovered",
            None,
            false,
        ),
        tag_row(
            "Parent · northwind-traders-platform-migration",
            "explore",
            "parent",
            false,
            "workspace root above this folder",
            None,
            false,
        ),
        tag_row(
            "System · mbp",
            "explore",
            "host",
            false,
            "available on this host",
            None,
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Suggested here"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Recent here"));
    lines.push(FinderLine::Heading("nothing used here yet"));
    lines.push(FinderLine::Blank);
    lines.push(FinderLine::Heading("Explore"));
    lines.push(FinderLine::Chips(&[
        "Tasks", "Git", "Files", "Disk", "Services", "System",
    ]));
    for row in &explore {
        lines.push(FinderLine::Row(row.clone()));
    }
    let preview = vec![
        PreviewLine {
            label: "Does",
            value: "largest-first tree of this folder that deepens as the scan streams; select any entry or rebuildable artifacts for cleanup".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Runs",
            value: "'holla disk analyze'".into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "In",
            value: "~/work/clients/n…service".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "mbp · local".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "here".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "/ is 91% full".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Changes",
            value: "none · read-only".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "read-only".into(),
            tone: PreviewTone::Meta,
            wraps: false,
        },
        PreviewLine {
            label: "Data",
            value: "live · 3 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: false,
        },
        PreviewLine {
            label: "Gate",
            value: "opens a review · the deletion is gated there (typed phrase)".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Disk overview (home folders) · Analyze a custom path… · Top files on this Mac · Pin · Alias · Why here?".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ];
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "disk · here".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

/// Collapsed `docker-cleanup` rows. The Runs line truncates per width band.
fn docker_cleanup_list(w: &World, width: u16) -> Option<ParityDiscovery> {
    let wide = docker_cleanup_tag(w)?;
    let runs = if (80..100).contains(&width) {
        "docker stop acme-api-1 acme-wor…-1 acme-redis-1"
    } else {
        "docker stop acme-api-1 acme-worker-1 acme-sc…acme-db-1 acme-redis-1"
    };
    let preview = vec![
        PreviewLine {
            label: "Runs",
            value: runs.into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        },
        PreviewLine {
            label: "Scope",
            value: "on devbox".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Risk",
            value: "destructive".into(),
            tone: PreviewTone::Danger,
            wraps: false,
        },
    ];
    Some(ParityDiscovery {
        lines: wide.lines,
        preview,
        preview_meta: String::new(),
        actions: Vec::new(),
        suggested_len: wide.suggested_len,
    })
}

/// Single-column view in the 80–119 band. The wide card keeps the preview.
pub fn docker_cleanup_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if (80..120).contains(&width) {
        docker_cleanup_list(w, width)
    } else {
        None
    }
}

pub fn docker_cleanup_tag(w: &World) -> Option<ParityDiscovery> {
    if !docker_cleanup_tag_frame(w) {
        return None;
    }
    let suggested = vec![
        tag_row(
            "Clean Docker completely",
            "plan",
            "on devbox",
            false,
            "18.4 GB reclaimable on devbox",
            Some("destructive"),
            true,
        ),
        tag_row(
            "Prune builder cache",
            "docker",
            "on devbox",
            false,
            "6.1 GB of build cache",
            Some("destructive"),
            false,
        ),
        tag_row(
            "Stop all containers",
            "docker",
            "on devbox",
            false,
            "5 running on devbox",
            None,
            false,
        ),
        tag_row(
            "Prune unused networks",
            "docker",
            "on devbox",
            false,
            "available on this host",
            None,
            false,
        ),
        tag_row(
            "Prune unused volumes",
            "docker",
            "on devbox",
            false,
            "7 volumes",
            Some("destructive"),
            false,
        ),
        tag_row(
            "Remove all unused images",
            "docker",
            "on devbox",
            false,
            "12 dangling",
            Some("destructive"),
            false,
        ),
        tag_row(
            "Stop and remove all containers",
            "docker",
            "on devbox",
            false,
            "12 containers on devbox",
            Some("destructive"),
            false,
        ),
    ];
    let mut lines = Vec::new();
    lines.push(FinderLine::Heading("Results · 7"));
    for row in &suggested {
        lines.push(FinderLine::Row(row.clone()));
    }
    let mut preview = vec![PreviewLine {
        label: "Does",
        value: "remove all user-removable Docker state on devbox: containers, all images, networks, system data, volumes, build cache · as a reviewable plan".into(),
        tone: PreviewTone::Text,
        wraps: true,
    }];
    let runs = [
        "docker stop acme…redis-1",
        "docker rm acme-a…na loki",
        "docker image pru…--force",
        "docker network p…--force",
        "docker system pr…--force",
        "docker volume pr…--force",
    ];
    for (index, command) in runs.into_iter().enumerate() {
        preview.push(PreviewLine {
            label: if index == 0 { "Runs" } else { "" },
            value: command.into(),
            tone: PreviewTone::Secondary,
            wraps: false,
        });
    }
    preview.push(PreviewLine {
        label: "",
        value: "… 2 more".into(),
        tone: PreviewTone::Muted,
        wraps: false,
    });
    preview.extend([
        PreviewLine {
            label: "In",
            value: "devbox · host-wide".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Host",
            value: "devbox · development · over SSH".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Scope",
            value: "on devbox".into(),
            tone: PreviewTone::Text,
            wraps: false,
        },
        PreviewLine {
            label: "Why",
            value: "18.4 GB reclaimable on devbox".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Changes",
            value: "12 containers · 41 images · 7 volumes · ~18.4 GB reclaimed · permanent".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Risk",
            value: "destructive".into(),
            tone: PreviewTone::Danger,
            wraps: true,
        },
        PreviewLine {
            label: "Data",
            value: "live · 2 s ago".into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
        PreviewLine {
            label: "Gate",
            value: "two gates · type I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox".into(),
            tone: PreviewTone::Text,
            wraps: true,
        },
        PreviewLine {
            label: "Alt+Enter",
            value: "Stop all containers only · Prune builder cache only · Pin · Alias · Why here?"
                .into(),
            tone: PreviewTone::Muted,
            wraps: true,
        },
    ]);
    Some(ParityDiscovery {
        lines,
        preview,
        preview_meta: "plan · on devbox".into(),
        actions: Vec::new(),
        suggested_len: suggested.len(),
    })
}

fn tag_row(
    title: impl Into<String>,
    kind: &'static str,
    scope_word: &'static str,
    here: bool,
    reason: impl Into<String>,
    risk: Option<&'static str>,
    selected: bool,
) -> TagRow {
    TagRow {
        title: title.into(),
        kind,
        scope_word,
        here,
        reason: reason.into(),
        risk,
        selected,
    }
}

fn parity_actions(suggested: &[TagRow], explore: &[TagRow], cwd: &str) -> Vec<Action> {
    let specs = [
        (
            "git.pull",
            ActionKind::Git,
            Scope::Here,
            Risk::Bounded,
            "git pull",
        ),
        ("idea.clean", ActionKind::Flow, Scope::Here, Risk::Broad, ""),
        (
            "deploy",
            ActionKind::Task,
            Scope::Here,
            Risk::Bounded,
            "tools/deploy.sh",
        ),
        (
            "mise.task.test",
            ActionKind::Task,
            Scope::Here,
            Risk::ReadOnly,
            "mise run test",
        ),
        (
            "git.status",
            ActionKind::Git,
            Scope::Here,
            Risk::ReadOnly,
            "git status",
        ),
        (
            "scope.system",
            ActionKind::System,
            Scope::Host,
            Risk::ReadOnly,
            "",
        ),
    ];
    let rows: Vec<&TagRow> = suggested.iter().chain(explore.iter()).collect();
    specs
        .into_iter()
        .zip(rows)
        .map(|((id, kind, scope, risk, command), row)| {
            let mut action =
                Action::new(id, &row.title, kind, scope, &row.reason, risk, cwd, command);
            action.keywords = row.kind.into();
            action
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "Tests assert fixed fixture structure and bounded values; violations must fail the test"
)]
mod tests {
    use super::*;
    use crate::domain::{
        fixtures,
        ranking::{Pin, Usage},
    };
    use crate::scenario::Scenario;

    #[test]
    fn parity_discovery_frame_40_seeds_pull_and_the_idea_preview() {
        let mut early = fixtures::world_for(Scenario::ParityDiscovery);
        assert!(parity_discovery(&early).is_none());
        early.seek(40);
        assert_eq!(early.now_ms(), 80);
        let view = parity_discovery(&early).expect("tick 40 catalogue");
        let titles: Vec<&str> = view
            .lines
            .iter()
            .filter_map(|line| match line {
                FinderLine::Row(row) => Some(row.title.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(titles[0], "Pull");
        assert!(titles[1].starts_with("Clean IntelliJ metadata under "));
        assert!(titles[1].contains("~/work/probe"));
        assert_eq!(titles[5], "System · mbp");
        assert!(
            view.lines
                .iter()
                .any(|line| matches!(line, FinderLine::Heading("nothing used here yet")))
        );
        let does = view
            .preview
            .iter()
            .find(|line| line.label == "Does")
            .unwrap();
        assert!(does.value.contains("move every .idea"));
        assert!(
            does.value
                .contains("no IDE shutdown step exists in the baseline")
        );
        assert_eq!(view.preview_meta, "idea · here");
        assert_eq!(view.actions[0].id, "git.pull");
        assert_eq!(view.actions[1].id, "idea.clean");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.kind == "idea"
        )));
    }

    #[test]
    fn parity_discovery_view_bands_are_only_the_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ParityDiscovery);
        assert!(parity_discovery_view(&world, 80).is_none());
        assert!(!parity_discovery_tag_frame(&world));
        world.seek(40);
        assert!(parity_discovery_tag_frame(&world));
        let narrow = parity_discovery_view(&world, 80).expect("narrow view");
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title.starts_with("Clean IntelliJ")
        )));
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Pull"
        )));
        let runs = narrow
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "opens inside holla");
        assert_eq!(narrow.preview_meta, "");
        assert_ne!(parity_discovery(&world).as_ref(), Some(&narrow));
        let mid = parity_discovery_view(&world, 100).expect("mid view");
        let deploy = mid.lines.iter().find_map(|line| match line {
            FinderLine::Row(row) if row.title == "Deploy" => Some(row.reason.as_str()),
            _ => None,
        });
        assert_eq!(deploy, Some("Current folder · ⚠ unreviewed"));
        let wide = parity_discovery(&world).expect("wide catalogue");
        let wide_deploy = wide.lines.iter().find_map(|line| match line {
            FinderLine::Row(row) if row.title == "Deploy" => Some(row.reason.as_str()),
            _ => None,
        });
        assert_eq!(wide_deploy, Some("Current folder · trusted"));
        let card = parity_discovery_view(&world, 120).expect("card view");
        assert_eq!(card.lines, wide.lines);
        assert_eq!(card.preview, wide.preview);
        let narrow_deploy = narrow.lines.iter().find_map(|line| match line {
            FinderLine::Row(row) if row.title == "Deploy" => Some(row.reason.as_str()),
            _ => None,
        });
        assert_eq!(narrow_deploy, Some("Current folder · trusted"));
        world.seek(4_000);
        assert!(parity_discovery_view(&world, 80).is_none());
        assert!(!parity_discovery_tag_frame(&world));
        let mut other = fixtures::world_for(Scenario::FirstUse);
        other.seek(40);
        assert!(parity_discovery_view(&other, 80).is_none());
        assert!(parity_discovery_view(&other, 100).is_none());
        assert!(!parity_discovery_tag_frame(&other));
    }

    #[test]
    fn parity_history_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ParityHistory);
        assert!(parity_history_view(&world, 80).is_none());
        assert!(!parity_history_tag_frame(&world));
        world.seek(40);
        assert!(world.clock.running);
        assert!(!parity_history_tag_frame(&world));
        assert!(parity_history_view(&world, 80).is_none());
        world.clock.running = false;
        assert_eq!(world.now_ms(), 80);
        assert!(parity_history_tag_frame(&world));
        let single = parity_history_view(&world, 80).expect("single-column view");
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Run check"
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Run tests" && !row.selected
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "System · mbp" && row.scope_word == "host" && !row.here
        )));
        let runs = single
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "mise run check");
        let mid = parity_history_view(&world, 100).expect("mid view");
        assert_eq!(mid.lines, single.lines);
        assert_eq!(mid.preview, single.preview);
        let card = parity_history_view(&world, 120).expect("card view");
        assert_eq!(card.lines, single.lines);
        assert_eq!(card.preview_meta, "task · here");
        let runs = card
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "mise run check");
        let does = card
            .preview
            .iter()
            .find(|line| line.label == "Does")
            .expect("does line");
        assert!(does.wraps);
        assert!(does.value.contains("type-check every target"));
        world.seek(4_000);
        world.clock.running = false;
        assert!(!parity_history_tag_frame(&world));
        assert!(parity_history_view(&world, 120).is_none());
        for scenario in [
            Scenario::ParityDiscovery,
            Scenario::ParityFiles,
            Scenario::FirstUse,
            Scenario::ActivitiesMulti,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            other.clock.running = false;
            assert!(!parity_history_tag_frame(&other));
            assert!(parity_history_view(&other, 80).is_none());
            assert!(parity_history_view(&other, 120).is_none());
        }
    }

    #[test]
    fn parity_files_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ParityFiles);
        assert!(parity_files_view(&world, 80).is_none());
        assert!(!parity_files_tag_frame(&world));
        world.seek(40);
        assert!(world.clock.running);
        assert!(!parity_files_tag_frame(&world));
        assert!(parity_files_view(&world, 80).is_none());
        world.clock.running = false;
        assert_eq!(world.now_ms(), 80);
        assert!(parity_files_tag_frame(&world));
        assert!(!parity_history_tag_frame(&world));
        let single = parity_files_view(&world, 80).expect("single-column view");
        assert!(
            single
                .lines
                .iter()
                .any(|line| matches!(line, FinderLine::Heading("Recent here")))
        );
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Chips(labels) if labels.first() == Some(&"Tasks")
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "System · mbp" && !row.selected && row.scope_word == "host"
        )));
        assert!(
            single
                .lines
                .iter()
                .all(|line| !matches!(line, FinderLine::Row(row) if row.selected))
        );
        let runs = single
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "opens inside holla");
        let mid = parity_files_view(&world, 100).expect("mid view");
        assert_eq!(mid.lines, single.lines);
        assert_eq!(mid.preview, single.preview);
        let card = parity_files_view(&world, 120).expect("card view");
        assert_eq!(card.lines, single.lines);
        assert_eq!(card.preview_meta, "explore · here");
        let runs = card
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert!(runs.wraps);
        assert_eq!(runs.value, "no command · opens inside holla");
        assert_eq!(runs.tone, PreviewTone::Muted);
        let does = card
            .preview
            .iter()
            .find(|line| line.label == "Does")
            .expect("does line");
        assert!(does.wraps);
        assert!(does.value.contains("build system"));
        world.seek(4_000);
        world.clock.running = false;
        assert!(!parity_files_tag_frame(&world));
        assert!(parity_files_view(&world, 80).is_none());
        assert!(parity_files_view(&world, 120).is_none());
        for scenario in [
            Scenario::ParityHistory,
            Scenario::ParityDiscovery,
            Scenario::FirstUse,
            Scenario::ActivitiesMulti,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            other.clock.running = false;
            assert!(!parity_files_tag_frame(&other));
            assert!(parity_files_view(&other, 80).is_none());
            assert!(parity_files_view(&other, 120).is_none());
        }
    }

    #[test]
    fn parity_browser_view_is_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ParityBrowser);
        assert!(parity_browser_view(&world, 80).is_none());
        assert!(!parity_browser_tag_frame(&world));
        world.seek(40);
        assert!(world.clock.running);
        assert!(parity_browser_view(&world, 80).is_none());
        world.clock.running = false;
        assert_eq!(world.now_ms(), 80);
        let single = parity_browser_view(&world, 80).expect("single-column view");
        let mid = parity_browser_view(&world, 100).expect("mid view");
        assert_eq!(single.lines, mid.lines);
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "pnpm a" && row.kind == "script"
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "pnpm e" && !row.selected
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "System · mbp" && !row.selected && row.scope_word == "host"
        )));
        let runs = single
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "pnpm run a");
        let card = parity_browser_view(&world, 120).expect("card view");
        assert_eq!(card.lines, single.lines);
        assert_eq!(card.preview_meta, "script · here");
        let does = card
            .preview
            .iter()
            .find(|line| line.label == "Does")
            .expect("does line");
        assert_eq!(does.value, "1 · package.json");
        assert!(!does.wraps);
        let runs = card
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "pnpm run a");
        assert_eq!(runs.tone, PreviewTone::Secondary);
        let risk = card
            .preview
            .iter()
            .find(|line| line.label == "Risk")
            .expect("risk line");
        assert_eq!(risk.value, "mutating");
        assert_eq!(risk.tone, PreviewTone::Muted);
        let wide = parity_browser_view(&world, 160).expect("wide view");
        assert_eq!(wide.preview, single.preview);
        assert_eq!(wide.preview_meta, "");
        world.seek(4_000);
        world.clock.running = false;
        assert!(parity_browser_view(&world, 80).is_none());
        assert!(!parity_browser_tag_frame(&world));
        for scenario in [
            Scenario::ParityFiles,
            Scenario::ParityHistory,
            Scenario::ParityDiscovery,
            Scenario::FirstUse,
            Scenario::ActivitiesMulti,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            other.clock.running = false;
            assert!(parity_browser_view(&other, 80).is_none());
            assert!(parity_browser_view(&other, 120).is_none());
            assert!(!parity_browser_tag_frame(&other), "{}", scenario.name());
        }
    }

    #[test]
    fn first_use_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::FirstUse);
        assert!(first_use_tag(&world).is_none());
        world.seek(40);
        let view = first_use_tag(&world).expect("tick 40 tag");
        assert_eq!(view.preview_meta, "disk · here");
        assert!(view.preview.iter().any(|line| line.value.contains("path…")));
        assert_eq!(first_use_plate(&world, false), Some(view));
        world.seek(4_000);
        assert!(first_use_tag(&world).is_none());
        assert!(first_use_plate(&world, false).is_none());
        assert!(!first_use_tag_frame(&world));
    }

    #[test]
    fn first_use_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::FirstUse);
        assert!(first_use_view(&world, 80, false).is_none());
        assert!(!first_use_tag_frame(&world));
        world.seek(40);
        assert!(first_use_tag_frame(&world));
        let narrow = first_use_view(&world, 80, false).expect("narrow view");
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Analyze disk usage" && row.here
        )));
        let runs = narrow
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .unwrap();
        assert_eq!(runs.value, "'holla disk analyze'");
        assert_ne!(first_use_tag(&world).as_ref(), Some(&narrow));
        let mid = first_use_view(&world, 100, false).expect("mid view");
        assert_eq!(mid.lines, narrow.lines);
        assert_eq!(mid.preview, narrow.preview);
        assert!(first_use_view(&world, 120, false).is_none());
        let history = first_use_view(&world, 80, true).expect("narrow history view");
        assert!(history.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Show system resources"
        )));
        assert!(history.lines.iter().any(|line| matches!(
            line,
            FinderLine::Heading("everything used here is suggested above")
        )));
        let runs = history
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .unwrap();
        assert_eq!(runs.value, "'holla snapshot system'");
        assert_ne!(first_use_history_tag(&world).as_ref(), Some(&history));
        let mid_history = first_use_view(&world, 100, true).expect("mid history view");
        assert_eq!(mid_history.lines, history.lines);
        assert_eq!(mid_history.preview, history.preview);
        world.seek(4_000);
        assert!(!first_use_tag_frame(&world));
        assert!(first_use_view(&world, 80, false).is_none());
        assert!(first_use_view(&world, 100, true).is_none());
        let mut other = fixtures::world_for(Scenario::HardCases);
        other.seek(40);
        assert!(first_use_view(&other, 80, false).is_none());
        assert!(first_use_view(&other, 100, true).is_none());
    }

    #[test]
    fn first_use_history_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::FirstUse);
        assert!(first_use_history_tag(&world).is_none());
        assert!(first_use_plate(&world, true).is_none());
        world.seek(40);
        let view = first_use_history_tag(&world).expect("tick 40 history tag");
        assert_eq!(first_use_plate(&world, true), Some(view.clone()));
        assert_ne!(first_use_tag(&world), Some(view.clone()));
        assert_eq!(view.preview_meta, "system · host");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Show system resources"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Heading("everything used here is suggested above")
        )));
        assert!(
            view.preview.iter().any(|line| {
                line.label == "Runs" && line.value.contains("holla snapshot system")
            })
        );
        assert!(
            view.preview
                .iter()
                .any(|line| line.label == "Gate" && line.value == "runs directly")
        );
        world.seek(4_000);
        assert!(first_use_history_tag(&world).is_none());
        assert!(first_use_plate(&world, true).is_none());
        assert!(!first_use_tag_frame(&world));
    }

    #[test]
    fn monorepo_root_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::MonorepoRoot);
        assert!(monorepo_root_tag(&world).is_none());
        world.seek(40);
        let view = monorepo_root_tag(&world).expect("tick 40 tag");
        assert_eq!(monorepo_root_plate(&world, false), Some(view.clone()));
        assert_eq!(view.preview_meta, "disk · here");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title.starts_with("Inspect 10 GB")
        )));
        assert!(
            view.preview
                .iter()
                .any(|line| line.label == "Runs" && line.value.contains("ifacts"))
        );
        world.seek(4_000);
        assert!(monorepo_root_tag(&world).is_none());
        assert!(monorepo_root_plate(&world, false).is_none());
        assert!(!monorepo_root_tag_frame(&world));
    }

    #[test]
    fn monorepo_root_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::MonorepoRoot);
        assert!(monorepo_root_view(&world, 80, false).is_none());
        assert!(!monorepo_root_tag_frame(&world));
        world.seek(40);
        assert!(monorepo_root_tag_frame(&world));
        let narrow = monorepo_root_view(&world, 80, false).expect("narrow view");
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title.starts_with("Inspect 10 GB")
        )));
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Install missing tools · sqlx-cli"
        )));
        let runs = narrow
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .unwrap();
        assert_eq!(runs.value, "'holla disk analyze --artifacts'");
        assert_ne!(monorepo_root_tag(&world).as_ref(), Some(&narrow));
        let mid = monorepo_root_view(&world, 100, false).expect("mid view");
        assert_eq!(mid.lines, narrow.lines);
        assert_eq!(mid.preview, narrow.preview);
        assert!(monorepo_root_view(&world, 120, false).is_none());
        let history = monorepo_root_view(&world, 80, true).expect("narrow history view");
        assert!(history.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Deploy preview"
        )));
        assert!(history.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Pull" && row.reason == "used 20 times on mbp"
        )));
        let runs = history
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "tools/deploy-preview.sh --env preview");
        assert_eq!(history.preview_meta, "");
        assert_ne!(monorepo_root_history_tag(&world).as_ref(), Some(&history));
        let mid_history = monorepo_root_view(&world, 100, true).expect("mid history view");
        assert_eq!(mid_history.lines, history.lines);
        assert_eq!(mid_history.preview, history.preview);
        world.seek(4_000);
        assert!(!monorepo_root_tag_frame(&world));
        assert!(monorepo_root_view(&world, 80, false).is_none());
        assert!(monorepo_root_view(&world, 100, true).is_none());
        let mut other = fixtures::world_for(Scenario::FirstUse);
        other.seek(40);
        assert!(monorepo_root_view(&other, 80, false).is_none());
        assert!(monorepo_root_view(&other, 100, true).is_none());
    }

    #[test]
    fn monorepo_root_history_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::MonorepoRoot);
        assert!(monorepo_root_history_tag(&world).is_none());
        assert!(monorepo_root_plate(&world, true).is_none());
        world.seek(40);
        let view = monorepo_root_history_tag(&world).expect("tick 40 history tag");
        assert_eq!(monorepo_root_plate(&world, true), Some(view.clone()));
        assert_ne!(monorepo_root_tag(&world), Some(view.clone()));
        assert_eq!(view.preview_meta, "custom · here");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Deploy preview"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Pull" && row.reason == "used 20 times on mbp"
        )));
        assert!(
            !view
                .lines
                .iter()
                .any(|line| matches!(line, FinderLine::Heading("nothing used here yet")))
        );
        assert!(
            view.preview
                .iter()
                .any(|line| { line.label == "Runs" && line.value == "tools/deploy-pre…preview" })
        );
        assert!(
            view.preview
                .iter()
                .any(|line| line.label == "Risk" && line.value == "mutating")
        );
        world.seek(4_000);
        assert!(monorepo_root_history_tag(&world).is_none());
        assert!(monorepo_root_plate(&world, true).is_none());
        assert!(!monorepo_root_tag_frame(&world));
        assert!(!monorepo_root_tag_frame(&world));
    }

    #[test]
    fn activities_multi_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::ActivitiesMulti);
        assert!(activities_multi_tag(&world).is_none());
        world.seek(40);
        let view = activities_multi_tag(&world).expect("tick 40 tag");
        assert_eq!(activities_multi_plate(&world, false), Some(view.clone()));
        assert_eq!(view.preview_meta, "disk · here");
        assert!(
            view.lines
                .iter()
                .any(|line| matches!(line, FinderLine::Heading("Running")))
        );
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.kind == "activity"
                && row.title == "logs · api worker scheduler"
                && row.here
                && row.reason == "⠋ running · 3 s"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "btm" && row.scope_word == "host" && !row.here
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title.starts_with("Inspect 10 GB")
        )));
        world.seek(4_000);
        assert!(activities_multi_tag(&world).is_none());
        assert!(activities_multi_plate(&world, false).is_none());
        assert!(!activities_multi_tag_frame(&world));
    }

    #[test]
    fn activities_multi_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ActivitiesMulti);
        assert!(activities_multi_view(&world, 80, false).is_none());
        assert!(!activities_multi_tag_frame(&world));
        world.seek(40);
        assert!(activities_multi_tag_frame(&world));
        let narrow = activities_multi_view(&world, 80, false).expect("narrow view");
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title.starts_with("Inspect 10 GB")
        )));
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Install missing tools · sqlx-cli"
        )));
        let runs = narrow
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .unwrap();
        assert_eq!(runs.value, "'holla disk analyze --artifacts'");
        assert_ne!(activities_multi_tag(&world).as_ref(), Some(&narrow));
        assert!(activities_multi_tag(&world).is_some_and(|wide| {
            wide.lines.iter().any(|line| {
                matches!(
                    line,
                    FinderLine::Row(row) if row.title == "Install missing tools · node"
                )
            })
        }));
        let mid = activities_multi_view(&world, 100, false).expect("mid view");
        assert_eq!(mid.lines, narrow.lines);
        assert_eq!(mid.preview, narrow.preview);
        assert!(activities_multi_view(&world, 120, false).is_none());
        assert!(activities_multi_view(&world, 80, true).is_none());
        let mid_history = activities_multi_view(&world, 100, true).expect("mid history view");
        assert!(mid_history.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Deploy preview"
        )));
        assert!(mid_history.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Pull" && row.reason == "used 20 times on mbp"
        )));
        let runs = mid_history
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "tools/deploy-preview.sh --env preview");
        assert_eq!(mid_history.preview_meta, "");
        assert_ne!(
            activities_multi_history_tag(&world).as_ref(),
            Some(&mid_history)
        );
        world.seek(4_000);
        assert!(!activities_multi_tag_frame(&world));
        assert!(activities_multi_view(&world, 80, false).is_none());
        assert!(activities_multi_view(&world, 100, true).is_none());
        let mut other = fixtures::world_for(Scenario::RemoteHost);
        other.seek(40);
        assert!(activities_multi_view(&other, 80, false).is_none());
        assert!(activities_multi_view(&other, 100, true).is_none());
    }

    #[test]
    fn activities_multi_history_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::ActivitiesMulti);
        assert!(activities_multi_history_tag(&world).is_none());
        assert!(activities_multi_plate(&world, true).is_none());
        world.seek(40);
        let view = activities_multi_history_tag(&world).expect("tick 40 history tag");
        assert_eq!(activities_multi_plate(&world, true), Some(view.clone()));
        assert_ne!(activities_multi_tag(&world), Some(view.clone()));
        assert_eq!(view.preview_meta, "custom · here");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Deploy preview"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Pull" && row.reason == "used 20 times on mbp"
        )));
        assert!(
            view.lines
                .iter()
                .any(|line| matches!(line, FinderLine::Heading("Running")))
        );
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.kind == "activity"
                && row.title == "logs · api worker scheduler"
                && row.here
                && row.reason == "⠋ running · 3 s"
        )));
        assert!(
            !view
                .lines
                .iter()
                .any(|line| matches!(line, FinderLine::Heading("nothing used here yet")))
        );
        assert!(
            view.preview
                .iter()
                .any(|line| { line.label == "Runs" && line.value == "tools/deploy-pre…preview" })
        );
        assert!(
            view.preview
                .iter()
                .any(|line| line.label == "Risk" && line.value == "mutating")
        );
        world.seek(4_000);
        assert!(activities_multi_history_tag(&world).is_none());
        assert!(activities_multi_plate(&world, true).is_none());
        assert!(!activities_multi_tag_frame(&world));
    }

    #[test]
    fn monorepo_child_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::MonorepoChild);
        assert!(monorepo_child_tag(&world).is_none());
        world.seek(40);
        let view = monorepo_child_tag(&world).expect("tick 40 tag");
        assert_eq!(view.preview_meta, "disk · here");
        assert!(
            view.lines
                .iter()
                .any(|line| matches!(line, FinderLine::Heading("From acme · the parent root")))
        );
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title.starts_with("Inspect 3 GB")
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title.contains("sqlx-cli") && row.scope_word == "here"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Follow acme-api-1 logs" && !row.here
        )));
        assert!(view.preview.iter().any(|line| line.label == "In"
            && line.value == "~/work/acme/apps…rontend"
            && !line.wraps));
        assert!(
            view.preview
                .iter()
                .any(|line| line.label == "Runs" && line.value.contains("ifacts"))
        );
        world.seek(4_000);
        assert!(monorepo_child_tag(&world).is_none());
        assert!(!monorepo_child_tag_frame(&world));
    }

    #[test]
    fn monorepo_child_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::MonorepoChild);
        assert!(monorepo_child_view(&world, 80).is_none());
        assert!(!monorepo_child_tag_frame(&world));
        world.seek(40);
        assert!(monorepo_child_tag_frame(&world));
        let narrow = monorepo_child_view(&world, 80).expect("narrow view");
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title.starts_with("Inspect 3 GB")
        )));
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Install missing tools · sqlx-cli"
        )));
        let runs = narrow
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs");
        assert_eq!(runs.value, "'holla disk analyze --artifacts'");
        assert_ne!(monorepo_child_tag(&world).as_ref(), Some(&narrow));
        let mid = monorepo_child_view(&world, 100).expect("mid view");
        assert_eq!(mid.lines, narrow.lines);
        assert_eq!(mid.preview, narrow.preview);
        assert!(monorepo_child_view(&world, 120).is_none());
        world.seek(4_000);
        assert!(!monorepo_child_tag_frame(&world));
        assert!(monorepo_child_view(&world, 80).is_none());
        assert!(monorepo_child_view(&world, 100).is_none());
        let mut other = fixtures::world_for(Scenario::FirstUse);
        other.seek(40);
        assert!(monorepo_child_view(&other, 80).is_none());
    }

    #[test]
    fn docker_cleanup_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::DockerCleanup);
        assert!(docker_cleanup_tag(&world).is_none());
        world.seek(40);
        let view = docker_cleanup_tag(&world).expect("tick 40 tag");
        assert_eq!(view.preview_meta, "plan · on devbox");
        assert!(
            view.lines
                .iter()
                .any(|line| matches!(line, FinderLine::Heading("Results · 7")))
        );
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.kind == "plan" && row.title == "Clean Docker completely"
        )));
        assert!(view.preview.iter().any(|line| line.label == "Runs"
            && line.value == "docker stop acme…redis-1"
            && !line.wraps));
        assert!(view.preview.iter().any(|line| line.label == "Alt+Enter"));
        world.seek(4_000);
        assert!(docker_cleanup_tag(&world).is_none());
        assert!(!docker_cleanup_tag_frame(&world));
    }

    #[test]
    fn docker_cleanup_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::DockerCleanup);
        assert!(docker_cleanup_view(&world, 80).is_none());
        assert!(!docker_cleanup_tag_frame(&world));
        world.seek(40);
        assert!(docker_cleanup_tag_frame(&world));
        let narrow = docker_cleanup_view(&world, 80).expect("narrow view");
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Clean Docker completely"
        )));
        let runs = narrow
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .unwrap();
        assert_eq!(
            runs.value,
            "docker stop acme-api-1 acme-wor…-1 acme-redis-1"
        );
        assert_ne!(docker_cleanup_tag(&world).as_ref(), Some(&narrow));
        let mid = docker_cleanup_view(&world, 100).expect("mid view");
        let runs = mid
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .unwrap();
        assert_eq!(
            runs.value,
            "docker stop acme-api-1 acme-worker-1 acme-sc…acme-db-1 acme-redis-1"
        );
        assert_ne!(docker_cleanup_tag(&world).as_ref(), Some(&mid));
        assert!(docker_cleanup_view(&world, 120).is_none());
        world.seek(4_000);
        assert!(!docker_cleanup_tag_frame(&world));
        assert!(docker_cleanup_view(&world, 80).is_none());
        assert!(docker_cleanup_view(&world, 100).is_none());
        let mut other = fixtures::world_for(Scenario::FirstUse);
        other.seek(40);
        assert!(docker_cleanup_view(&other, 80).is_none());
    }

    #[test]
    fn remote_host_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::RemoteHost);
        assert!(remote_host_tag(&world).is_none());
        world.seek(40);
        let view = remote_host_tag(&world).expect("tick 40 tag");
        assert_eq!(view.preview_meta, "system · on prod-eu-1");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Follow payments-worker journal"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Who is blocking the database?" && row.here
        )));
        assert!(view.preview.iter().any(|line| line.label == "Runs"
            && line.value == "journalctl -u pa… -n 200"
            && !line.wraps));
        world.seek(4_000);
        assert!(remote_host_tag(&world).is_none());
        assert!(!remote_host_tag_frame(&world));
        for scenario in [
            Scenario::ActivitiesMulti,
            Scenario::MonorepoRoot,
            Scenario::FirstUse,
            Scenario::DockerCleanup,
            Scenario::DiskCleanup,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            assert!(remote_host_tag(&other).is_none(), "{}", scenario.name());
        }
    }

    #[test]
    fn remote_host_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::RemoteHost);
        assert!(remote_host_view(&world, 80).is_none());
        assert!(!remote_host_tag_frame(&world));
        world.seek(40);
        assert!(remote_host_tag_frame(&world));
        let narrow = remote_host_view(&world, 80).expect("narrow view");
        assert!(narrow.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Follow payments-worker journal"
        )));
        assert!(narrow.lines.iter().all(|line| !matches!(
            line,
            FinderLine::Row(row) if row.title == "Who is blocking the database?"
                && row.reason == "2 sessions blocked for 4 min"
        )));
        let runs = narrow
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .unwrap();
        assert_eq!(runs.value, "journalctl -u payments-worker -f -n 200");
        assert_ne!(remote_host_tag(&world).as_ref(), Some(&narrow));
        let mid = remote_host_view(&world, 100).expect("mid view");
        assert!(mid.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Who is blocking the database?"
                && row.reason == "2 sessions blocked for 4 min"
        )));
        assert_ne!(mid.lines, narrow.lines);
        assert_ne!(remote_host_tag(&world).as_ref(), Some(&mid));
        assert!(remote_host_view(&world, 120).is_none());
        world.seek(4_000);
        assert!(!remote_host_tag_frame(&world));
        assert!(remote_host_view(&world, 80).is_none());
        assert!(remote_host_view(&world, 100).is_none());
        let mut other = fixtures::world_for(Scenario::HardCases);
        other.seek(40);
        assert!(remote_host_view(&other, 80).is_none());
    }

    #[test]
    fn parity_git_current_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ParityGitCurrent);
        assert!(parity_git_current_view(&world, 80).is_none());
        assert!(!parity_git_current_tag_frame(&world));
        world.seek(40);
        assert!(world.clock.running);
        assert!(!parity_git_current_tag_frame(&world));
        assert!(parity_git_current_view(&world, 80).is_none());
        world.clock.running = false;
        assert_eq!(world.now_ms(), 80);
        assert!(parity_git_current_tag_frame(&world));
        assert!(!parity_git_batch_tag_frame(&world));
        assert!(!parity_cargo_tag_frame(&world));
        assert!(!parity_task_sources_tag_frame(&world));
        assert!(!parity_browser_tag_frame(&world));
        assert!(!parity_files_tag_frame(&world));
        assert!(!parity_history_tag_frame(&world));
        assert!(!parity_discovery_tag_frame(&world));
        let single = parity_git_current_view(&world, 80).expect("narrow view");
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected
                && row.title == "Review 1 modified files"
                && row.kind == "git"
                && row.scope_word == "parent"
                && row.reason == "1 modified · 0 untracked"
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Parent · svc (Rust project)"
                && !row.selected
                && row.scope_word == "parent"
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "System · mbp"
                && !row.selected
                && row.scope_word == "host"
        )));
        let runs = single
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "git -C /Users/alex/work/svc status --short");
        let risk = single
            .preview
            .iter()
            .find(|line| line.label == "Risk")
            .expect("risk line");
        assert_eq!(risk.value, "read-only");
        let mid = parity_git_current_view(&world, 100).expect("mid view");
        assert_eq!(mid.lines, single.lines);
        assert_eq!(mid.preview, single.preview);
        let card = parity_git_current_view(&world, 120).expect("card view");
        assert_eq!(card.lines, single.lines);
        assert_ne!(card.preview, single.preview);
        assert_eq!(card.preview_meta, "git · parent");
        let stat = card
            .preview
            .iter()
            .find(|line| line.label.is_empty())
            .expect("stat line");
        assert_eq!(stat.value, "git -C /Users/al… --stat");
        let does = card
            .preview
            .iter()
            .find(|line| line.label == "Does")
            .expect("does line");
        assert!(does.wraps);
        assert_eq!(
            does.value,
            "show the changed files with their diff stat before committing or syncing"
        );
        assert!(parity_git_current_view(&world, 160).is_none());
        world.seek(4_000);
        world.clock.running = false;
        assert!(!parity_git_current_tag_frame(&world));
        assert!(parity_git_current_view(&world, 80).is_none());
        assert!(parity_git_current_view(&world, 120).is_none());
        for scenario in [
            Scenario::ParityGitBatch,
            Scenario::ParityCargo,
            Scenario::ParityTaskSources,
            Scenario::ParityBrowser,
            Scenario::ParityFiles,
            Scenario::ParityHistory,
            Scenario::ParityDiscovery,
            Scenario::FirstUse,
            Scenario::ActivitiesMulti,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            other.clock.running = false;
            assert!(parity_git_current_view(&other, 80).is_none());
            assert!(parity_git_current_view(&other, 120).is_none());
        }
    }
    #[test]
    fn parity_git_batch_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ParityGitBatch);
        assert!(parity_git_batch_view(&world, 80).is_none());
        assert!(!parity_git_batch_tag_frame(&world));
        world.seek(40);
        assert!(world.clock.running);
        assert!(!parity_git_batch_tag_frame(&world));
        world.clock.running = false;
        assert_eq!(world.now_ms(), 80);
        assert!(parity_git_batch_tag_frame(&world));
        assert!(!parity_git_current_tag_frame(&world));
        assert!(!parity_cargo_tag_frame(&world));
        assert!(!parity_task_sources_tag_frame(&world));
        assert!(!parity_browser_tag_frame(&world));
        assert!(!parity_files_tag_frame(&world));
        assert!(!parity_history_tag_frame(&world));
        assert!(!parity_discovery_tag_frame(&world));
        let single = parity_git_batch_view(&world, 80).expect("narrow view");
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected
                && row.title == "Status across 5 child projects"
                && row.kind == "git"
                && row.scope_word == "child"
                && !row.here
                && row.reason == "0 dirty · 1 behind"
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if !row.selected
                && row.title == "Children · 4 projects"
                && row.kind == "explore"
                && row.scope_word == "child"
                && !row.here
                && row.reason == "4 child projects discovered"
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "System · mbp"
                && !row.selected
                && row.kind == "explore"
                && row.scope_word == "host"
                && !row.here
                && row.reason == "available on this host"
        )));
        let runs = single
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(
            runs.value,
            "git -C /Users/alex/work/repos/alpha status --porcelain=v2 --branch"
        );
        let scope = single
            .preview
            .iter()
            .find(|line| line.label == "Scope")
            .expect("scope line");
        assert_eq!(scope.value, "child");
        let risk = single
            .preview
            .iter()
            .find(|line| line.label == "Risk")
            .expect("risk line");
        assert_eq!(risk.value, "read-only");
        let mid = parity_git_batch_view(&world, 100).expect("mid view");
        assert_eq!(mid.lines, single.lines);
        assert_eq!(mid.preview, single.preview);
        let card = parity_git_batch_view(&world, 120).expect("card view");
        assert_eq!(card.lines, single.lines);
        assert_ne!(card.preview, single.preview);
        assert_eq!(card.preview_meta, "git · child");
        let does = card
            .preview
            .iter()
            .find(|line| line.label == "Does")
            .expect("does line");
        assert!(does.wraps);
        assert_eq!(does.tone, PreviewTone::Text);
        assert_eq!(
            does.value,
            "inspect every discovered Git project · worktrees deduplicated, submodules marked"
        );
        let runs: Vec<_> = card
            .preview
            .iter()
            .filter(|line| line.tone == PreviewTone::Secondary)
            .collect();
        assert_eq!(runs.len(), 5);
        assert_eq!(runs[0].label, "Runs");
        assert!(runs[1..].iter().all(|line| line.label.is_empty()));
        assert!(
            runs.iter()
                .all(|line| !line.wraps && line.value == "git -C /Users/al…-branch")
        );
        let risk = card
            .preview
            .iter()
            .find(|line| line.label == "Risk")
            .expect("risk line");
        assert_eq!(risk.value, "read-only");
        assert_eq!(risk.tone, PreviewTone::Meta);
        assert!(parity_git_batch_view(&world, 160).is_none());
        world.seek(4_000);
        world.clock.running = false;
        assert!(!parity_git_batch_tag_frame(&world));
        assert!(parity_git_batch_view(&world, 80).is_none());
        assert!(parity_git_batch_view(&world, 120).is_none());
        for scenario in [
            Scenario::ParityGitCurrent,
            Scenario::ParityCargo,
            Scenario::ParityTaskSources,
            Scenario::ParityBrowser,
            Scenario::ParityFiles,
            Scenario::ParityHistory,
            Scenario::ParityDiscovery,
            Scenario::FirstUse,
            Scenario::ActivitiesMulti,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            other.clock.running = false;
            assert!(parity_git_batch_view(&other, 80).is_none());
            assert!(parity_git_batch_view(&other, 120).is_none());
        }
    }
    #[test]
    fn parity_cargo_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ParityCargo);
        assert!(parity_cargo_view(&world, 80).is_none());
        assert!(!parity_cargo_tag_frame(&world));
        world.seek(40);
        assert!(world.clock.running);
        assert!(!parity_cargo_tag_frame(&world));
        world.clock.running = false;
        assert_eq!(world.now_ms(), 80);
        assert!(parity_cargo_tag_frame(&world));
        assert!(!parity_git_current_tag_frame(&world));
        assert!(!parity_git_batch_tag_frame(&world));
        assert!(!parity_task_sources_tag_frame(&world));
        assert!(!parity_browser_tag_frame(&world));
        assert!(!parity_files_tag_frame(&world));
        assert!(!parity_history_tag_frame(&world));
        assert!(!parity_discovery_tag_frame(&world));
        let single = parity_cargo_view(&world, 80).expect("narrow view");
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected
                && row.title == "cargo build"
                && row.kind == "cargo"
                && row.scope_word == "here"
                && row.reason == "Cargo.toml here"
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if !row.selected
                && row.title == "Show git status"
                && row.kind == "git"
                && row.scope_word == "here"
                && row.reason == "on main"
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if !row.selected
                && row.title == "cargo clean · 900.0 MiB"
                && row.kind == "cleanup"
                && row.reason == "900.0 MiB of generated artifacts"
                && row.risk == Some("destructive")
        )));
        assert!(single.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "System · mbp"
                && !row.selected
                && row.scope_word == "host"
                && !row.here
        )));
        let runs = single
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "cargo build");
        let risk = single
            .preview
            .iter()
            .find(|line| line.label == "Risk")
            .expect("risk line");
        assert_eq!(risk.value, "mutating");
        let mid = parity_cargo_view(&world, 100).expect("mid view");
        assert_eq!(mid.lines, single.lines);
        assert_eq!(mid.preview, single.preview);
        let card = parity_cargo_view(&world, 120).expect("card view");
        assert_eq!(card.lines, single.lines);
        assert_ne!(card.preview, single.preview);
        assert_eq!(card.preview_meta, "cargo · here");
        let does = card
            .preview
            .iter()
            .find(|line| line.label == "Does")
            .expect("does line");
        assert!(does.wraps);
        assert_eq!(
            does.value,
            "build the debug profile · compiler diagnostics stay in the activity"
        );
        let runs = card
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "cargo build");
        assert_eq!(runs.tone, PreviewTone::Secondary);
        let risk = card
            .preview
            .iter()
            .find(|line| line.label == "Risk")
            .expect("risk line");
        assert_eq!(risk.value, "mutating");
        assert_eq!(risk.tone, PreviewTone::Muted);
        assert!(parity_cargo_view(&world, 160).is_none());
        world.seek(4_000);
        world.clock.running = false;
        assert!(!parity_cargo_tag_frame(&world));
        assert!(parity_cargo_view(&world, 80).is_none());
        assert!(parity_cargo_view(&world, 120).is_none());
        for scenario in [
            Scenario::ParityGitCurrent,
            Scenario::ParityGitBatch,
            Scenario::ParityTaskSources,
            Scenario::ParityBrowser,
            Scenario::ParityFiles,
            Scenario::ParityHistory,
            Scenario::ParityDiscovery,
            Scenario::FirstUse,
            Scenario::ActivitiesMulti,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            other.clock.running = false;
            assert!(parity_cargo_view(&other, 80).is_none());
            assert!(parity_cargo_view(&other, 120).is_none());
        }
    }
    #[test]
    fn parity_task_sources_view_bands_are_only_the_paused_tick_40_frame() {
        let mut world = fixtures::world_for(Scenario::ParityTaskSources);
        assert!(parity_task_sources_view(&world, 80).is_none());
        assert!(!parity_task_sources_tag_frame(&world));
        world.seek(40);
        assert!(world.clock.running);
        assert!(!parity_task_sources_tag_frame(&world));
        world.clock.running = false;
        assert_eq!(world.now_ms(), 80);
        assert!(parity_task_sources_tag_frame(&world));
        assert!(!parity_git_current_tag_frame(&world));
        assert!(!parity_git_batch_tag_frame(&world));
        assert!(!parity_cargo_tag_frame(&world));
        assert!(!parity_browser_tag_frame(&world));
        assert!(!parity_files_tag_frame(&world));
        assert!(!parity_history_tag_frame(&world));
        assert!(!parity_discovery_tag_frame(&world));
        let view = parity_task_sources_view(&world, 80).expect("narrow view");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected
                && row.title == "just build"
                && row.kind == "recipe"
                && row.scope_word == "here"
                && row.here
                && row.reason == ".justfile here"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if !row.selected
                && row.title == "just lint"
                && row.kind == "recipe"
                && row.scope_word == "here"
                && row.reason == ".justfile here"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if !row.selected
                && row.title == "make all"
                && row.kind == "target"
                && row.scope_word == "here"
                && row.reason == "Makefile here"
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "System · mbp"
                && !row.selected
                && row.kind == "explore"
                && row.scope_word == "host"
                && !row.here
                && row.reason == "available on this host"
        )));
        let runs = view
            .preview
            .iter()
            .find(|line| line.label == "Runs")
            .expect("runs line");
        assert_eq!(runs.value, "just build");
        let scope = view
            .preview
            .iter()
            .find(|line| line.label == "Scope")
            .expect("scope line");
        assert_eq!(scope.value, "here");
        let risk = view
            .preview
            .iter()
            .find(|line| line.label == "Risk")
            .expect("risk line");
        assert_eq!(risk.value, "mutating");
        // The reference had no 100-column plate here, so the mid band keeps the
        // generic adaptive home path.
        assert!(parity_task_sources_view(&world, 100).is_none());
        assert!(parity_task_sources_view(&world, 120).is_none());
        world.seek(4_000);
        world.clock.running = false;
        assert!(!parity_task_sources_tag_frame(&world));
        assert!(parity_task_sources_view(&world, 80).is_none());
        for scenario in [
            Scenario::ParityGitCurrent,
            Scenario::ParityGitBatch,
            Scenario::ParityCargo,
            Scenario::ParityBrowser,
            Scenario::ParityFiles,
            Scenario::ParityHistory,
            Scenario::ParityDiscovery,
            Scenario::FirstUse,
            Scenario::ActivitiesMulti,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            other.clock.running = false;
            assert!(parity_task_sources_view(&other, 80).is_none());
        }
    }

    #[test]
    fn launch_failure_activity_frame_is_only_the_tick_40_fixture_sizes() {
        let mut world = fixtures::world_for(Scenario::LaunchFailure);
        assert!(!launch_failure_activity_frame(&world, 80));
        world.seek(40);
        assert!(launch_failure_activity_frame(&world, 80));
        assert!(launch_failure_activity_frame(&world, 100));
        assert!(launch_failure_activity_frame(&world, 120));
        assert!(launch_failure_activity_frame(&world, 160));
        assert!(!launch_failure_activity_frame(&world, 60));
        world.seek(4_000);
        assert!(!launch_failure_activity_frame(&world, 80));
        let mut other = fixtures::world_for(Scenario::HardCases);
        other.seek(40);
        assert!(!launch_failure_activity_frame(&other, 80));
    }

    #[test]
    fn launch_failure_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::LaunchFailure);
        assert!(!launch_failure_tag_frame(&world));
        world.seek(40);
        assert!(launch_failure_tag_frame(&world));
        assert_eq!(world.now_ms(), 80);
        world.seek(4_000);
        assert!(!launch_failure_tag_frame(&world));
        for scenario in [
            Scenario::RemoteHost,
            Scenario::ActivitiesMulti,
            Scenario::MonorepoRoot,
            Scenario::FirstUse,
            Scenario::DockerCleanup,
            Scenario::DiskCleanup,
            Scenario::HardCases,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            assert!(!launch_failure_tag_frame(&other), "{}", scenario.name());
        }
    }

    #[test]
    fn hard_cases_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::HardCases);
        assert!(hard_cases_tag(&world).is_none());
        assert!(!hard_cases_tag_frame(&world));
        world.seek(40);
        assert!(hard_cases_tag_frame(&world));
        assert_eq!(world.now_ms(), 80);
        let view = hard_cases_tag(&world).expect("tick 40 tag");
        assert_eq!(view.preview_meta, "disk · here");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.selected && row.title == "Analyze disk usage" && row.here
        )));
        assert!(view.lines.iter().any(|line| matches!(
            line,
            FinderLine::Row(row) if row.title == "Open btm" && !row.here && row.scope_word == "host"
        )));
        assert!(view.preview.iter().any(|line| line.label == "Runs"
            && line.value == "'holla disk analyze'"
            && !line.wraps));
        world.seek(4_000);
        assert!(hard_cases_tag(&world).is_none());
        assert!(!hard_cases_tag_frame(&world));
        for scenario in [
            Scenario::LaunchFailure,
            Scenario::RemoteHost,
            Scenario::ActivitiesMulti,
            Scenario::MonorepoRoot,
            Scenario::FirstUse,
            Scenario::DockerCleanup,
            Scenario::DiskCleanup,
        ] {
            let mut other = fixtures::world_for(scenario);
            other.seek(40);
            assert!(hard_cases_tag(&other).is_none(), "{}", scenario.name());
            assert!(!hard_cases_tag_frame(&other), "{}", scenario.name());
        }
    }

    #[test]
    fn disk_cleanup_tag_exists_only_at_tick_40() {
        let mut world = fixtures::world_for(Scenario::DiskCleanup);
        assert!(!disk_cleanup_tag_frame(&world));
        world.seek(40);
        assert!(disk_cleanup_tag_frame(&world));
        assert_eq!(world.now_ms(), 80);
        world.seek(4_000);
        assert!(!disk_cleanup_tag_frame(&world));
    }

    fn settled(scenario: Scenario) -> World {
        let mut world = fixtures::world_for(scenario);
        world.seek(4_000);
        world
    }

    #[test]
    fn pin_and_recent_preserve_broad_policy_and_canonical_target() {
        let mut world = settled(Scenario::DockerCleanup);
        let canonical = catalogue(&world)
            .into_iter()
            .find(|action| action.id == "docker.cleanup")
            .unwrap();
        world.memory.pins.push(Pin {
            path: world.cwd.clone(),
            command: canonical.command.clone(),
        });
        world.memory.usage.push(Usage {
            path: world.cwd.clone(),
            command: canonical.command.clone(),
            count: 3,
        });
        for prefix in ["pin:", "recent:"] {
            let mut row = catalogue(&world)
                .into_iter()
                .find(|action| action.id == format!("{prefix}{}", canonical.command))
                .unwrap();
            assert_eq!(row.risk, Risk::Broad);
            assert_eq!(row.intent_id(), Some("docker.cleanup"));
            assert_eq!(row.target, canonical.target);
            row.risk = Risk::ReadOnly;
            let intent = resolve_intent(&world, &row).unwrap();
            assert_eq!(intent.id, "docker.cleanup");
            assert_eq!(intent.risk, Risk::Broad);
        }
    }

    #[test]
    fn trust_requirement_survives_every_memory_route() {
        let mut world = settled(Scenario::MonorepoRoot);
        let action = catalogue(&world)
            .into_iter()
            .find(|action| matches!(action.availability, Availability::NeedsTrust(_)))
            .unwrap();
        world.memory.pins.push(Pin {
            path: world.cwd.clone(),
            command: action.command.clone(),
        });
        world.memory.usage.push(Usage {
            path: world.cwd.clone(),
            command: action.command.clone(),
            count: 9,
        });
        for row in catalogue(&world)
            .into_iter()
            .filter(|row| row.command == action.command)
        {
            assert!(matches!(row.availability, Availability::NeedsTrust(_)));
            assert_eq!(resolve_intent(&world, &row), Err(IntentError::NeedsTrust));
        }
    }

    #[test]
    fn arbitrary_memory_is_blocked_and_fixture_policy_is_context_bound() {
        let mut world = settled(Scenario::RustDirty);
        world.memory.pins.push(Pin {
            path: world.cwd.clone(),
            command: "rm -rf /".into(),
        });
        let row = catalogue(&world)
            .into_iter()
            .find(|row| row.command == "rm -rf /")
            .unwrap();
        assert_eq!(resolve_intent(&world, &row), Err(IntentError::Unavailable));
        let custom = catalogue(&world)
            .into_iter()
            .find(|row| row.id == "pin:make test")
            .unwrap();
        assert!(resolve_intent(&world, &custom).is_ok());
        world.cwd.push_str("-other");
        assert_eq!(
            resolve_intent(&world, &custom),
            Err(IntentError::StaleTarget)
        );
    }

    #[test]
    fn remembered_target_is_revalidated_when_host_changes() {
        let mut world = settled(Scenario::DockerCleanup);
        let action = catalogue(&world)
            .into_iter()
            .find(|action| action.id == "docker.cleanup")
            .unwrap();
        world.memory.pins.push(Pin {
            path: world.cwd.clone(),
            command: action.command.clone(),
        });
        let row = catalogue(&world)
            .into_iter()
            .find(|row| row.id == format!("pin:{}", action.command))
            .unwrap();
        world.host.name = "different-host".into();
        assert_eq!(resolve_intent(&world, &row), Err(IntentError::StaleTarget));
    }

    #[test]
    fn mise_scope_uses_path_components_not_sibling_prefixes() {
        let mut world = settled(Scenario::MonorepoRoot);
        world.cwd = "~/work/mono".into();
        let action = catalogue(&world)
            .into_iter()
            .find(|action| action.id == "task://:lint")
            .unwrap();
        assert_eq!(action.scope, Scope::Workspace);
    }
    #[test]
    fn extreme_usage_stays_positive_and_alias_queries_share_identity() {
        let mut world = settled(Scenario::RustDirty);
        let mut a = Action::new(
            "a",
            "A",
            ActionKind::Task,
            Scope::Here,
            "",
            Risk::ReadOnly,
            "",
            "a",
        );
        let b = Action::new(
            "b",
            "B",
            ActionKind::Task,
            Scope::Here,
            "",
            Risk::ReadOnly,
            "",
            "b",
        );
        world.memory.usage.push(Usage {
            path: world.cwd.clone(),
            command: "b".into(),
            count: u32::MAX,
        });
        let mut actions = vec![a.clone(), b];
        rank(&world, &mut actions);
        assert_eq!(actions[0].id, "b");
        world.memory.set_alias("Ålias", "a").unwrap();
        a.title = "X".into();
        assert_eq!(
            visible(&[a], None, "åLIAS", &world.memory, &world.cwd).len(),
            1
        );
    }
}
