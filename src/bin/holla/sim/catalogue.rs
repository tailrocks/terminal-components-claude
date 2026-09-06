//! Catalogue: derive the ranked action list from discovered fixture truth.
//! Ranking = scope ring weight + pins/usage memory + live urgency; every
//! score component is visible in the row's reason (ranking is inspectable).

use crate::domain::action::{Action, ActionKind, Availability, Risk, Scope};
use crate::domain::docker::Health;
use crate::domain::human_bytes;
use crate::domain::mise::Trust;
use crate::domain::ranking::Memory;
use crate::sim::world::{Domain, World};

/// One ranked section on home.
pub struct Section {
    pub name: &'static str,
    pub actions: Vec<Action>,
}

/// Derive every known action for the current world state. Domains still
/// discovering contribute nothing yet; failed domains contribute a blocked
/// note row instead of silence.
pub fn catalogue(w: &World) -> Vec<Action> {
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

/// Suggested / Recent / Explore with Suggested holding the top-ranked four.
/// Pins and exact aliases always lead Suggested; live urgency can elevate a
/// nonlocal row, but never above them.
pub fn sections(w: &World) -> Vec<Section> {
    let mut all = catalogue(w);
    rank(w, &mut all);
    // ranking is inspectable: the alias an action carries shows as a reason
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

/// Score every action: ring weight, pin/alias/usage memory, live urgency.
/// Sort is total and deterministic: score, then kind, then title.
pub fn rank(w: &World, actions: &mut [Action]) {
    let score = |a: &Action| -> i32 {
        let mut s = a.scope.weight();
        // pins are the strongest user intent: they must outrank an alias
        // plus live urgency combined (90 + 30)
        if w.memory.pin_at(&w.cwd, &a.command) {
            s += 150;
        }
        if w.memory.alias_for(&a.command).is_some() {
            s += 90;
        }
        s += (w.memory.usage_at(&w.cwd, &a.command) as i32).min(10) * 2;
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

/// Live-urgency elevation (§9 rank 3): something is wrong or stale NOW.
fn urgent(a: &Action) -> bool {
    a.reason.contains("unhealthy")
        || a.reason.contains("modified")
        || a.reason.contains("behind")
        || a.reason.contains("untrusted")
        || a.reason.contains("% full")
        || a.reason.contains("detached")
}

// ------------------------------------------------------------ derivation

fn mise_actions(w: &World, out: &mut Vec<Action>) {
    if !w.discovered(Domain::Mise) {
        return;
    }
    let Some(mise) = &w.mise else { return };
    for t in &mise.tasks {
        let workdir = t
            .defined_in
            .rsplit_once('/')
            .map(|(d, _)| d.to_owned())
            .unwrap_or_else(|| w.cwd.clone());
        // Ring: file in cwd subtree → Here; namespaced child → Project;
        // parent ecosystem visible from a child cwd → Workspace.
        let scope = if t.defined_in.starts_with(&w.cwd) && !t.namespaced() {
            Scope::Here
        } else if t.namespaced() && workdir.starts_with(&w.cwd) {
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
        a.workdir = workdir.clone();
        a.scope_label = scope_place(&w.cwd, &workdir);
        a.keywords = t.command.clone();
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
        a.workdir = git.root.clone();
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
        // the bulk plan: per-child chains, per-child primary branches
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
    status.workdir = git.root.clone();
    out.push(status);
}

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
    for c in d.unhealthy() {
        // restarting a service on a production host is broad: two gates
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
        a.scope_label = w.host.name.clone();
        out.push(a);
    }
    let reclaim = d.reclaimable_bytes();
    if reclaim >= GB {
        let mut a = Action::new(
            "docker.cleanup",
            "Clean up Docker data",
            ActionKind::Plan,
            Scope::Host,
            &format!(
                "{} builder cache · {} reclaimable",
                human_bytes(d.build_cache_bytes),
                human_bytes(reclaim)
            ),
            Risk::Broad,
            &w.host.name,
            "docker system prune -a --volumes",
        );
        a.scope_label = w.host.name.clone();
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
    status.scope_label = w.host.name.clone();
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
        logs.scope_label = w.host.name.clone();
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
    let pct = disk.used_percent();
    let reclaim = disk.reclaimable_bytes();
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
    a.scope_label = w.host.name.clone();
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

/// Host flows that exist regardless of folder content (Explore never
/// empty): btm handoff and the disk-usage flow.
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
    btm.scope_label = w.host.name.clone();
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

/// Recent here: per-path usage from ranking memory.
fn recent_actions(w: &World, out: &mut Vec<Action>) {
    let mut usage: Vec<_> = w.memory.usage.iter().filter(|u| u.path == w.cwd).collect();
    usage.sort_by_key(|u| std::cmp::Reverse(u.count));
    for u in usage {
        out.push(Action::new(
            &format!("recent:{}", u.command),
            &u.command,
            ActionKind::Task,
            Scope::Here,
            &format!("used {} in this project", times(u.count)),
            Risk::ReadOnly,
            &w.cwd,
            &u.command,
        ));
    }
    for pin in &w.memory.pins {
        if pin.path != w.cwd {
            continue;
        }
        let mut a = Action::new(
            &format!("pin:{}", pin.command),
            &pin.command,
            ActionKind::Task,
            Scope::Here,
            "pinned here",
            Risk::ReadOnly,
            &w.cwd,
            &pin.command,
        );
        a.scope_label = "pin".into();
        out.push(a);
    }
}

// ---------------------------------------------------------------- helpers

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

/// Ring of a filesystem path relative to cwd.
pub fn path_scope(cwd: &str, path: &str) -> Scope {
    if path == cwd || path.starts_with(&format!("{cwd}/")) {
        Scope::Here
    } else if path.starts_with('/') {
        Scope::Host
    } else {
        Scope::Personal
    }
}

/// Short place label for a scope (`apps/frontend` under the monorepo).
fn scope_place(cwd: &str, workdir: &str) -> String {
    if workdir == cwd {
        return String::new();
    }
    workdir
        .strip_prefix(&format!("{cwd}/"))
        .map(str::to_owned)
        .or_else(|| {
            // child cwd: show the anchor dir name of the parent ecosystem
            workdir.rsplit('/').next().map(str::to_owned)
        })
        .unwrap_or_default()
}

/// Filter + grouping for the home list: hidden rows are gone, scope filter
/// first, then the lowercase substring query across title, reason, scope
/// and command. An alias query matches the row of the command it expands
/// to (`gs` finds `git status`). A hidden row resurfaces when the query is
/// its exact command — that is the only way to reach Unhide/Reset.
pub fn visible(
    actions: &[Action],
    scope: Option<Scope>,
    query: &str,
    memory: &Memory,
    cwd: &str,
) -> Vec<Action> {
    let q = query.trim().to_lowercase();
    let expansion = memory
        .aliases
        .iter()
        .find(|a| a.alias.to_lowercase() == q)
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
