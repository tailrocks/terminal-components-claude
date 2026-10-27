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
    pub empty_note: Option<&'static str>,
    pub chips: Option<&'static [&'static str]>,
}

pub fn catalogue(w: &World) -> Vec<Action> {
    catalogue_with_history(w, history_enabled())
}

fn catalogue_with_history(w: &World, history: bool) -> Vec<Action> {
    if let Some(view) = parity_discovery(w) {
        return view.actions;
    }
    let mut out = vec![];
    if w.scenario == Scenario::FirstUse {
        first_use_actions(w, &mut out);
    }
    mise_actions(w, &mut out);
    git_actions(w, &mut out);
    docker_actions(w, &mut out);
    disk_actions(w, &mut out);
    pg_actions(w, &mut out);
    ssh_actions(w, &mut out);
    github_actions(w, &mut out);
    debian_actions(w, &mut out);
    flow_actions(w, &mut out);
    out.push(crate::port_args::catalogue_action(w));
    recent_actions(w, &mut out, history);
    out
}

pub fn sections(w: &World) -> Vec<Section> {
    sections_with_history(w, history_enabled())
}

pub fn sections_with_history(w: &World, history: bool) -> Vec<Section> {
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
                empty_note: None,
                chips: None,
            },
            Section {
                name: "Explore",
                actions: explore,
                empty_note: None,
                chips: None,
            },
        ];
    }
    if w.scenario == Scenario::FirstUse {
        return first_use_sections(w, history);
    }
    let all = home_actions_with_history(w, history);
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
            empty_note: None,
            chips: None,
        });
    }
    if !recent.is_empty() {
        out.push(Section {
            name: "Recent here",
            actions: recent,
            empty_note: None,
            chips: None,
        });
    }
    if !explore.is_empty() {
        out.push(Section {
            name: "Explore",
            actions: explore,
            empty_note: None,
            chips: None,
        });
    }
    out
}

pub fn rank(w: &World, actions: &mut [Action]) {
    rank_with_history(w, actions, history_enabled());
}

pub fn home_actions_with_history(w: &World, history: bool) -> Vec<Action> {
    if let Some(view) = parity_discovery(w) {
        return view.actions;
    }
    let mut actions = catalogue_with_history(w, history);
    rank_with_history(w, &mut actions, history);
    for action in &mut actions {
        if let Some(name) = w.memory.alias_for(&action.command) {
            action.reason = format!("{} · alias {name}", action.reason);
        }
    }
    actions
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "Closed scope weights and capped memory and recommendation scores bound the total"
)]
pub fn rank_with_history(w: &World, actions: &mut [Action], history: bool) {
    let score = |a: &Action| -> i32 {
        let mut s = a.scope.weight();

        if w.memory.pin_at(&w.cwd, &a.command) {
            s += 150;
        }
        if w.memory.alias_for(&a.command).is_some() {
            s += 90;
        }
        if history {
            s += w.memory.usage_at(&w.cwd, &a.command).min(10) as i32 * 2;
            let count = w.memory.host_usage_count(&w.host.name, &a.id).min(5);
            s += count as i32 * 40;
            let recent = w
                .memory
                .host_usage_last_used_ms(&w.host.name, &a.id)
                .is_some_and(|last| {
                    let age = w.now_ms().saturating_sub(last);
                    (0..=7 * 24 * 60 * 60 * 1_000).contains(&age)
                });
            if recent {
                s += 30;
            }
        }
        if w.scenario == Scenario::FirstUse {
            match a.id.as_str() {
                "system.monitor"
                    if w.system
                        .as_ref()
                        .is_some_and(|system| system.btm_installed && pressure_active(system)) =>
                {
                    s += 75;
                }
                "scope.system" => s += 45,
                _ => {}
            }
        }
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

fn first_use_sections(w: &World, history: bool) -> Vec<Section> {
    let all = home_actions_with_history(w, history);
    let suggested_ids = ["system.resources", "disk.usage", "system.monitor"];
    let suggested: Vec<Action> = all
        .iter()
        .filter(|action| suggested_ids.contains(&action.id.as_str()))
        .cloned()
        .collect();
    let has_recent_projection =
        history && all.iter().any(|action| first_use_recently_used(w, action));
    let recent = if history {
        all.iter()
            .filter(|action| {
                first_use_recently_used(w, action)
                    && !suggested.iter().any(|suggested| {
                        action.id == suggested.id
                            || (action.id.starts_with("recent:")
                                && action.command == suggested.command)
                    })
            })
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let recent_ids: Vec<&str> = recent.iter().map(|action| action.id.as_str()).collect();
    let explore = all
        .iter()
        .filter(|action| {
            action.id == "scope.system"
                || (action.id.starts_with("pin:") && !recent_ids.contains(&action.id.as_str()))
        })
        .cloned()
        .collect();
    vec![
        Section {
            name: "Suggested here",
            actions: suggested,
            empty_note: None,
            chips: None,
        },
        Section {
            name: "Recent here",
            empty_note: recent.is_empty().then_some(if has_recent_projection {
                "everything used here is suggested above"
            } else {
                "nothing used here yet"
            }),
            actions: recent,
            chips: None,
        },
        Section {
            name: "Explore",
            actions: explore,
            empty_note: None,
            chips: Some(&["Tasks", "Git", "Files", "Disk", "Services", "System"]),
        },
    ]
}

fn first_use_recently_used(world: &World, action: &Action) -> bool {
    let host_recent = world.memory.host_usage_count(&world.host.name, &action.id) > 0
        && world
            .memory
            .host_usage_last_used_ms(&world.host.name, &action.id)
            .is_some_and(|last| {
                let age = world.now_ms().saturating_sub(last);
                (0..=30 * 24 * 60 * 60 * 1_000).contains(&age)
            });
    host_recent || action.id.starts_with("recent:")
}

fn first_use_actions(w: &World, out: &mut Vec<Action>) {
    let host = w.host.name.as_str();
    let cwd = w.cwd.as_str();
    let Some(system) = &w.system else { return };
    let resource_reason = if system.pressure_minutes > 0 {
        format!(
            "CPU {}% for {} min",
            system.cpu_pct, system.pressure_minutes
        )
    } else if let Some((some, full)) = system
        .pressure
        .filter(|(some, full)| *some > 0 || *full > 0)
    {
        format!("CPU pressure some {some}% · full {full}%")
    } else {
        format!(
            "CPU {}% · memory {:.1} / {:.0} GB",
            system.cpu_pct,
            f64::from(w.host.metrics.mem_used_mb) / 1_024.0,
            f64::from(w.host.metrics.mem_total_mb) / 1_024.0
        )
    };
    let mut resources = Action::new(
        "system.resources",
        "Show system resources",
        ActionKind::System,
        Scope::Host,
        &resource_reason,
        Risk::ReadOnly,
        host,
        "holla snapshot system",
    );
    resources.scope_label = host.to_owned();
    resources.workdir = cwd.to_owned();
    resources.keywords =
        "system resources cpu memory load pressure what uses cpu what uses memory slow".into();
    out.push(resources);

    if let Some(disk) = &w.disk {
        let pct = rounded_disk_percent(disk.used_bytes, disk.total_bytes);
        let mut action = Action::new(
            "disk.usage",
            "Analyze disk usage",
            ActionKind::Flow,
            Scope::Here,
            &format!("/ is {pct}% full"),
            Risk::ReadOnly,
            cwd,
            "holla disk analyze",
        );
        action.workdir = cwd.to_owned();
        action.keywords = "disk usage analyze space full".into();
        out.push(action);
    }

    if system.btm_installed {
        let mut action = Action::new(
            "system.monitor",
            "Open btm",
            ActionKind::Flow,
            Scope::Host,
            if system.pressure_minutes > 0 {
                "sustained pressure · continuous view fits"
            } else if pressure_active(system) {
                "pressure active · continuous view fits"
            } else {
                "btm installed · continuous view available"
            },
            Risk::ReadOnly,
            host,
            "btm",
        );
        action.scope_label = host.to_owned();
        action.workdir = cwd.to_owned();
        action.long_running = true;
        action.keywords = "btm bottom monitor top htop processes system monitor".into();
        out.push(action);
    }

    let mut scope = Action::new(
        "scope.system",
        &format!("System · {host}"),
        ActionKind::System,
        Scope::Host,
        "available on this host",
        Risk::ReadOnly,
        host,
        "",
    );
    scope.scope_label = host.to_owned();
    scope.workdir = cwd.to_owned();
    scope.keywords = "system host scope".into();
    out.push(scope);

    let mut processes = Action::new(
        "system.processes",
        "Show process tree",
        ActionKind::System,
        Scope::Host,
        &format!("{} noted processes", system.processes.len()),
        Risk::ReadOnly,
        host,
        "holla snapshot processes",
    );
    processes.scope_label = host.to_owned();
    processes.workdir = cwd.to_owned();
    processes.keywords = "process processes tree ps pid".into();
    out.push(processes);
}

pub fn first_use_action_preview(world: &World, action: &Action, history: bool) -> ActionPreview {
    let source = match action.id.as_str() {
        "disk.usage" => Some(Domain::Filesystem),
        "scope.system" | "system.resources" | "system.processes" | "system.monitor" => {
            Some(Domain::System)
        }
        _ => None,
    };
    let data = source.map_or_else(
        || format!("live · {} s ago", world.now_ms().max(0) / 1_000),
        |domain| match world
            .discovery
            .iter()
            .find(|discovery| discovery.domain == domain)
        {
            Some(discovery) if discovery.done => {
                let age = world.now_ms().saturating_sub(discovery.at_ms).max(0);
                format!("live · {} s ago", age / 1_000)
            }
            Some(discovery) if discovery.failed => "unavailable".to_owned(),
            _ => "still discovering…".to_owned(),
        },
    );
    let mut lines = Vec::new();
    let mut add = |label, value: String, tone, wraps| {
        lines.push(PreviewLine {
            label,
            value,
            tone,
            wraps,
        });
    };
    match action.id.as_str() {
        "system.resources" => add(
            "Does",
            "CPU, load, memory, swap, pressure, disk and network snapshot; then hand off to btm for continuous monitoring".into(),
            PreviewTone::Text,
            true,
        ),
        "system.processes" => add(
            "Does",
            "processes by CPU and memory".into(),
            PreviewTone::Text,
            true,
        ),
        "system.monitor" => add(
            "Does",
            "continuous host monitoring through the simulated activity lifecycle".into(),
            PreviewTone::Text,
            true,
        ),
        "disk.usage" => add(
            "Does",
            "largest-first tree of this folder that deepens as the scan streams".into(),
            PreviewTone::Text,
            true,
        ),
        "scope.system" => add(
            "Does",
            "show actions for the current host".into(),
            PreviewTone::Text,
            false,
        ),
        _ => add("Does", action.reason.clone(), PreviewTone::Text, true),
    }
    if action.command.is_empty() {
        add(
            "Runs",
            "no command · opens inside holla".into(),
            PreviewTone::Secondary,
            true,
        );
    } else {
        add(
            "Runs",
            format!("'{}'", action.command),
            PreviewTone::Secondary,
            false,
        );
    }
    add(
        "In",
        if action.scope == Scope::Host {
            format!("{} · host-wide", action.target)
        } else if action.workdir.is_empty() {
            action.target.clone()
        } else {
            action.workdir.clone()
        },
        PreviewTone::Text,
        true,
    );
    if !action.scope_label.is_empty() {
        add(
            "Host",
            format!("{} · {}", action.scope_label, world.host.kind.label()),
            PreviewTone::Text,
            false,
        );
    }
    add(
        "Scope",
        action.scope.label().to_owned(),
        PreviewTone::Text,
        false,
    );
    let mut reason = action.reason.clone();
    if history {
        let count = world.memory.host_usage_count(&world.host.name, &action.id);
        if count > 0 {
            reason.push_str(&format!(" · used {} on {}", times(count), world.host.name));
        }
    }
    add("Why", reason, PreviewTone::Text, true);
    add(
        "Changes",
        if action.risk == Risk::ReadOnly {
            "none · read-only".into()
        } else {
            action.changes()
        },
        PreviewTone::Text,
        true,
    );
    add(
        "Risk",
        match action.risk {
            Risk::ReadOnly => "read-only",
            Risk::Bounded => "bounded",
            Risk::Broad => "broad",
        }
        .into(),
        PreviewTone::Meta,
        false,
    );
    add("Data", data, PreviewTone::Muted, false);
    if action.id == "system.resources" {
        add("Gate", "runs directly".into(), PreviewTone::Muted, false);
        let actions = home_actions_with_history(world, history);
        let alternatives = ["system.monitor", "system.processes"]
            .into_iter()
            .filter_map(|id| actions.iter().find(|candidate| candidate.id == id))
            .map(|candidate| candidate.title.as_str())
            .collect::<Vec<_>>();
        if !alternatives.is_empty() {
            add(
                "Alt+Enter",
                format!("{} · Pin · Alias · Why here?", alternatives.join(" · ")),
                PreviewTone::Muted,
                true,
            );
        }
    } else if action.id == "system.processes" {
        add(
            "Gate",
            "opens the process snapshot · nothing executed".into(),
            PreviewTone::Muted,
            true,
        );
    } else if action.id == "system.monitor" {
        add(
            "Gate",
            "starts a simulated activity · no host process launched".into(),
            PreviewTone::Muted,
            true,
        );
    }
    let kind = match action.kind {
        ActionKind::Task => "task",
        ActionKind::Git => "git",
        ActionKind::Plan => "plan",
        ActionKind::Connect => "connect",
        ActionKind::Flow => "flow",
        ActionKind::Clone => "clone",
        ActionKind::System => "system",
    };
    ActionPreview {
        lines,
        meta: format!("{kind} · {}", action.scope.label()),
    }
}

fn rounded_disk_percent(used_bytes: u64, total_bytes: u64) -> u32 {
    if total_bytes == 0 {
        return 0;
    }
    let numerator = u128::from(used_bytes).saturating_mul(100);
    let denominator = u128::from(total_bytes);
    u32::try_from(numerator.saturating_add(denominator / 2) / denominator).unwrap_or(u32::MAX)
}

fn urgent(a: &Action) -> bool {
    a.reason.contains("unhealthy")
        || a.reason.contains("modified")
        || a.reason.contains("behind")
        || a.reason.contains("untrusted")
        || a.reason.contains("% full")
        || a.reason.contains("detached")
}

fn pressure_active(system: &crate::world::SystemSnapshot) -> bool {
    system.pressure_minutes > 0
        || system
            .pressure
            .is_some_and(|(some, full)| some > 0 || full > 0)
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
    if w.scenario == Scenario::FirstUse {
        return;
    }
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

fn recent_actions(w: &World, out: &mut Vec<Action>, history: bool) {
    let canonical = out.clone();
    if history {
        let mut usage: Vec<_> = w.memory.usage.iter().filter(|u| u.path == w.cwd).collect();
        usage.sort_by_key(|u| std::cmp::Reverse(u.count));
        for u in usage {
            let action = memory_action(w, &canonical, &u.command);
            out.push(action.remembered(
                format!("recent:{}", u.command),
                format!("used {} in this project", times(u.count)),
                false,
            ));
        }
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

    Secondary,

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPreview {
    pub lines: Vec<PreviewLine>,
    pub meta: String,
}

pub fn first_use_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::FirstUse && w.now_ms() == 80
}

pub fn history_enabled() -> bool {
    !std::env::var_os("HOLLA_NO_HISTORY").is_some_and(|value| value == "1")
}

pub fn monorepo_root_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::MonorepoRoot && w.now_ms() == 80
}

pub fn monorepo_child_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::MonorepoChild && w.now_ms() == 80
}

pub fn activities_multi_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ActivitiesMulti && w.now_ms() == 80
}

pub fn docker_cleanup_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::DockerCleanup && w.now_ms() == 80
}

pub fn disk_cleanup_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::DiskCleanup && w.now_ms() == 80
}

pub fn remote_host_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::RemoteHost && w.now_ms() == 80
}

pub fn launch_failure_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::LaunchFailure && w.now_ms() == 80
}

pub fn launch_failure_activity_frame(w: &World, width: u16) -> bool {
    launch_failure_tag_frame(w) && (80..=160).contains(&width)
}

pub fn hard_cases_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::HardCases && w.now_ms() == 80
}

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

pub fn parity_discovery_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityDiscovery && w.now_ms() == 80
}

pub fn parity_discovery_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_discovery_tag_frame(w) {
        return None;
    }
    let mut view = parity_discovery(w)?;
    if width < 120 {
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

pub fn parity_history_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityHistory && !w.clock.running && w.now_ms() == 80
}

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

pub fn parity_browser_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityBrowser && !w.clock.running && w.now_ms() == 80
}

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

pub fn parity_git_current_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityGitCurrent && !w.clock.running && w.now_ms() == 80
}

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

pub fn parity_git_batch_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityGitBatch && !w.clock.running && w.now_ms() == 80
}

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

pub fn parity_task_sources_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityTaskSources && !w.clock.running && w.now_ms() == 80
}

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

pub fn parity_task_sources_view(w: &World, width: u16) -> Option<ParityDiscovery> {
    if !parity_task_sources_tag_frame(w) {
        return None;
    }
    if width == 80 {
        return parity_task_sources_rows();
    }
    None
}

pub fn parity_cargo_tag_frame(w: &World) -> bool {
    w.scenario == Scenario::ParityCargo && !w.clock.running && w.now_ms() == 80
}

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

pub fn first_use_plate(w: &World, history: bool) -> Option<ParityDiscovery> {
    if history {
        first_use_history_tag(w)
    } else {
        first_use_tag(w)
    }
}

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

pub fn monorepo_root_plate(w: &World, history: bool) -> Option<ParityDiscovery> {
    if history {
        monorepo_root_history_tag(w)
    } else {
        monorepo_root_tag(w)
    }
}

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

pub fn activities_multi_plate(w: &World, history: bool) -> Option<ParityDiscovery> {
    if history {
        activities_multi_history_tag(w)
    } else {
        activities_multi_tag(w)
    }
}

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
