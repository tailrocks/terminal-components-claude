//! The catalogue: turns the world into rows. Capability (a domain being
//! present) becomes actions (items); live state adds reasons that make some
//! of them recommendations. Every row carries scope, risk, confirmation and
//! the exact command it stands for.

use crate::domain::action::{
    ArgSpec, Confirmation, Freshness, Item, Kind, Launch, ResultType, Risk, Signal,
};
use crate::domain::context::{Scope, ScopeTag};
use crate::sim::world::{SourceState, World};

/// Where a directory sits relative to the working directory.
fn tag_for_dir(w: &World, dir: &str) -> ScopeTag {
    let cwd = &w.location.cwd;
    if dir == cwd {
        ScopeTag::here(cwd)
    } else if cwd.starts_with(&format!("{dir}/")) {
        ScopeTag::parent(dir)
    } else if dir.starts_with(&format!("{cwd}/")) {
        ScopeTag::child(dir)
    } else if let Some(ws) = &w.location.workspace
        && dir.starts_with(&format!("{}/", ws.root))
    {
        // a sibling project under the same workspace root: reached through
        // the parent, runs in its own folder
        ScopeTag {
            direction: Scope::Parent,
            defined_at: dir.to_owned(),
            runs_in: dir.to_owned(),
            word: "sibling".into(),
        }
    } else {
        ScopeTag::host(&w.host.name)
    }
}

fn host_tag(w: &World) -> ScopeTag {
    if w.host.remote {
        ScopeTag::remote(&w.host.name)
    } else {
        ScopeTag::host(&w.host.name)
    }
}

fn freshness(w: &World, source: &str) -> Freshness {
    match w.source_state(source) {
        SourceState::Loading => Freshness::Loading,
        SourceState::Failed(r) => Freshness::Unavailable(r),
        SourceState::Done => {
            let done_at = w
                .sources
                .iter()
                .find(|s| s.name == source)
                .map(|s| s.done_at)
                .unwrap_or(0);
            let age_ms = (w.tick.saturating_sub(done_at) as i64) * crate::sim::world::TICK_MS;
            match source {
                // gh answers from its own cache; say so
                "github" => Freshness::Cached {
                    age_ms: age_ms + 40_000,
                },
                // an unreadable folder makes child discovery partial
                "children" if w.disk.partial_reason.is_some() => Freshness::Partial {
                    loaded: w.location.children.len(),
                    total: w.location.children.len() + 2,
                },
                _ => Freshness::Live { age_ms },
            }
        }
    }
}

fn script_for_task(w: &World, namespaced: &str, name: &str, dir: &str) -> String {
    let fail = w.scenario == crate::scenario::Scenario::LaunchFailure;
    let holla = dir.ends_with("/holla");
    match name {
        "dev" if dir.ends_with("apps/frontend") => {
            if fail {
                "task:frontend-dev-fail".into()
            } else {
                "task:frontend-dev".into()
            }
        }
        "dev" if dir.ends_with("services/api") => "task:api-dev".into(),
        "test" if dir.ends_with("apps/frontend") => "task:tests-frontend".into(),
        "migrate" => "task:migrate".into(),
        "test" if holla || dir.ends_with("services/api") => "task:tests".into(),
        "lint" if holla => "task:clippy".into(),
        "setup" => "task:setup".into(),
        "up" => "task:up".into(),
        "dev" if dir.ends_with("/acme") => "task:eco".into(),
        _ => format!("generic:mise run {namespaced}"),
    }
}

/// Fill usage, pins, aliases and hidden state from memory.
fn apply_memory(w: &World, mut item: Item) -> Item {
    let cwd = &w.location.cwd;
    let project_root = w.location.project.as_ref().map(|p| p.root.as_str());
    for u in &w.memory.usage {
        if u.item != item.id {
            continue;
        }
        match &u.path {
            Some(p) if p == cwd || Some(p.as_str()) == project_root => {
                item.used_here += u.count;
                item.last_used_secs = Some(item.last_used_secs.unwrap_or(0).max(u.last_secs));
            }
            Some(_) => {}
            None if u.host == w.host.name => item.used_anywhere += u.count,
            None => {}
        }
    }
    if item.used_here > 0 {
        let text = if item.used_here == 1 {
            "used once here".to_owned()
        } else {
            format!("used {} times here", item.used_here)
        };
        item.reasons
            .push(crate::domain::action::Reason::new(Signal::LocalUse, text));
    } else if item.used_anywhere > 0 {
        item.reasons.push(crate::domain::action::Reason::new(
            Signal::GlobalUse,
            format!("used {} times on {}", item.used_anywhere, w.host.name),
        ));
    }
    for (a, id) in &w.memory.aliases {
        if id == &item.id && !item.aliases.contains(a) {
            item.aliases.push(a.clone());
        }
    }
    item.pinned = w
        .memory
        .pins
        .iter()
        .any(|(p, id)| p == cwd && id == &item.id);
    item.hidden = w
        .memory
        .hidden
        .iter()
        .any(|(p, id)| p == cwd && id == &item.id);
    if item.pinned {
        item.reasons.push(crate::domain::action::Reason::new(
            Signal::Pin,
            "pinned here",
        ));
    }
    if let Some((_, tool)) = w
        .memory
        .preferred_tools
        .iter()
        .find(|(id, _)| id == &item.id)
    {
        item.preferred_tool = Some(tool.clone());
    }
    item
}

pub fn build(w: &World) -> Vec<Item> {
    let mut items: Vec<Item> = vec![];
    explore(w, &mut items);
    mise_items(w, &mut items);
    git_items(w, &mut items);
    rust_items(w, &mut items);
    docker_items(w, &mut items);
    system_items(w, &mut items);
    disk_items(w, &mut items);
    pg_items(w, &mut items);
    ssh_items(w, &mut items);
    service_items(w, &mut items);
    workflow_items(w, &mut items);
    file_items(w, &mut items);
    activity_items(w, &mut items);
    items.into_iter().map(|i| apply_memory(w, i)).collect()
}

fn explore(w: &World, out: &mut Vec<Item>) {
    let here = ScopeTag::here(&w.location.cwd);
    let mut ex = |id: &str, label: &str, group: &'static str, aliases: &[&str], summary: &str| {
        out.push(
            Item::new(id, label, Kind::Explore, ResultType::Flow, here.clone())
                .summary(summary)
                .aliases(aliases)
                .group("Explore")
                .launch(Launch::Group(group))
                .reason(Signal::Default, "stable entry point"),
        );
    };
    ex(
        "explore.tasks",
        "Tasks",
        "Tasks",
        &["t"],
        "project tasks from mise and the build system, here and around",
    );
    ex(
        "explore.git",
        "Git",
        "Git",
        &["g"],
        "worktree state, sync, branches, bulk operations across children",
    );
    ex(
        "explore.files",
        "Files",
        "Files",
        &["f"],
        "nearby files, configs and logs",
    );
    ex(
        "explore.disk",
        "Disk",
        "Disk",
        &["d"],
        "usage analysis and stack-aware cleanup",
    );
    ex(
        "explore.services",
        "Services",
        "Services",
        &["s"],
        "containers, Compose, databases and system services",
    );
    ex(
        "explore.system",
        "System",
        "System",
        &["y"],
        "resources, processes, monitors, upgrades",
    );
    // scope directions
    if let Some(p) = &w.location.workspace {
        out.push(
            Item::new(
                "scope.parent",
                &format!("Parent · {} ({})", p.name, p.kind.label()),
                Kind::Explore,
                ResultType::Resource,
                ScopeTag::parent(&p.root),
            )
            .summary(&format!(
                "show what the workspace root at {} defines for this folder",
                w.location.short(&p.root)
            ))
            .keywords(&["parent", "ancestor", "workspace", "monorepo", "root"])
            .group("Explore")
            .launch(Launch::Scope(Scope::Parent))
            .reason(Signal::Context, "workspace root above this folder"),
        );
    } else if let Some(p) = &w.location.project
        && p.root != w.location.cwd
    {
        out.push(
            Item::new(
                "scope.parent",
                &format!("Parent · {} ({})", p.name, p.kind.label()),
                Kind::Explore,
                ResultType::Resource,
                ScopeTag::parent(&p.root),
            )
            .summary(&format!(
                "show what the project root at {} defines",
                w.location.short(&p.root)
            ))
            .keywords(&["parent", "ancestor", "project root"])
            .group("Explore")
            .launch(Launch::Scope(Scope::Parent))
            .reason(Signal::Context, "project root above this folder"),
        );
    }
    if !w.location.children.is_empty() {
        let n = w.location.children.len();
        let names: Vec<&str> = w
            .location
            .children
            .iter()
            .take(3)
            .map(|c| c.name.as_str())
            .collect();
        out.push(
            Item::new(
                "scope.children",
                &format!("Children · {n} projects"),
                Kind::Explore,
                ResultType::Resource,
                ScopeTag::child(&w.location.cwd),
            )
            .summary(&format!(
                "bounded child projects below this folder: {}{}",
                names.join(", "),
                if n > 3 { ", …" } else { "" }
            ))
            .keywords(&["children", "child", "descendants", "projects"])
            .group("Explore")
            .launch(Launch::Scope(Scope::Children))
            .freshness(freshness(w, "children"))
            .reason(Signal::Context, &format!("{n} child projects discovered")),
        );
    }
    // `System` is already a chip on the Explore row; the row form would
    // name it twice. Kept for query matches only (hidden from Explore).
    out.push(
        Item::new(
            "scope.system",
            &format!("System · {}", w.host.name),
            Kind::Explore,
            ResultType::Resource,
            host_tag(w),
        )
        .summary(&format!(
            "host-wide capabilities on {} · {} · {}",
            w.host.name,
            w.host.os.label(),
            w.host.role.label()
        ))
        .keywords(&["system", "host", "machine", "everything"])
        .group("Explore")
        .launch(Launch::Scope(Scope::System))
        .reason(Signal::Default, "available on this host"),
    );
}

fn mise_items(w: &World, out: &mut Vec<Item>) {
    if !w.mise.installed {
        return;
    }
    let git = w.git_here();
    for cfg in &w.mise.configs {
        let trusted = cfg.trusted || w.trusted_now.contains(&cfg.path);
        for t in &cfg.tasks {
            let tag = tag_for_dir(w, &t.dir);
            let id = format!("mise.task.{}", t.namespaced);
            let label = match tag.direction {
                Scope::Here => format!("Run {}", t.name),
                _ => format!("Run {}", t.namespaced),
            };
            let label = match t.name.as_str() {
                "dev" if tag.direction == Scope::Parent => "Start development ecosystem".to_owned(),
                "up" if tag.direction == Scope::Parent => "Start required containers".to_owned(),
                "setup" if tag.direction == Scope::Parent => "Run workspace setup".to_owned(),
                "test" if tag.direction == Scope::Here => "Run tests".to_owned(),
                "dev" if tag.direction == Scope::Here => "Start dev server".to_owned(),
                "lint" if tag.direction == Scope::Here => "Run lint".to_owned(),
                _ => label,
            };
            let mut item = Item::new(&id, &label, Kind::Mise, ResultType::Action, tag.clone())
                .summary(&format!(
                    "{} · runs in {}",
                    t.description,
                    w.location.short(&t.dir)
                ))
                .commands(&[&format!("mise run {}", t.namespaced)])
                .effects(&[&format!("runs `{}` in {}", t.run, w.location.short(&t.dir))])
                .keywords(&[&t.name, &t.namespaced, "task", "mise", &t.description])
                .group("Tasks")
                .freshness(freshness(w, "mise"))
                .launch(Launch::Activity {
                    script: script_for_task(w, &t.namespaced, &t.name, &t.dir),
                })
                .reason(
                    Signal::Context,
                    &format!("defined by {}", w.location.short(&cfg.path)),
                )
                .alt("Show the task definition", "file.mise")
                .alt("Dry run", &format!("{id}.dry"));
            if !t.depends.is_empty() {
                item = item.effects(&[
                    &format!("runs `{}` in {}", t.run, w.location.short(&t.dir)),
                    &format!("after {}", t.depends.join(", ")),
                ]);
            }
            // `test` is the preferred test action here, never a fixed id
            if t.name == "test" && tag.direction == Scope::Here {
                item = item.aliases(&["test"]);
            }
            if !trusted {
                item = item
                    .confirm(Confirmation::Trust {
                        config: w.location.short(&cfg.path),
                    })
                    .risk(Risk::Mutating);
            }
            if t.name == "test" && tag.direction == Scope::Here && git.is_some_and(|g| g.dirty()) {
                let n = git.map(|g| g.changed() + g.untracked).unwrap_or(0);
                item = item.reason(
                    Signal::Urgency,
                    &format!("{n} files changed since the last run"),
                );
                item = item.args(vec![
                    ArgSpec::text("filter", "test name substring", "")
                        .help("passes through to nextest"),
                    ArgSpec::choice("profile", &["default", "ci"], 0),
                ]);
                item = item.alt("Run with cargo test instead", "cargo.test");
            }
            if t.name == "dev" {
                item = item.risk(Risk::Mutating).effects(&[
                    &format!(
                        "starts a long-running server in {}",
                        w.location.short(&t.dir)
                    ),
                    "becomes a named activity",
                ]);
            }
            if t.name == "up" {
                item = item.risk(Risk::Mutating);
            }
            if t.name == "setup" {
                item = item
                    .risk(Risk::Mutating)
                    .effects(&["installs tools and dependencies for every project"]);
            }
            out.push(item);
        }
    }
    // tools
    let outdated = w.mise.outdated();
    if !outdated.is_empty() {
        let names: Vec<String> = outdated
            .iter()
            .map(|t| {
                format!(
                    "{} {} → {}",
                    t.name,
                    t.active.clone().unwrap_or_default(),
                    t.latest
                )
            })
            .collect();
        out.push(
            Item::new(
                "mise.outdated",
                &format!("Upgrade {} outdated mise tools", outdated.len()),
                Kind::Mise,
                ResultType::Recommendation,
                host_tag(w),
            )
            .summary("show current, requested and latest versions, then upgrade the selected tools")
            .commands(&[
                "mise outdated --json",
                "mise upgrade --dry-run",
                "mise upgrade --interactive",
            ])
            .effects(&names.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .keywords(&[
                "mise", "tools", "outdated", "upgrade", "versions", "node", "python",
            ])
            .group("System")
            .risk(Risk::Mutating)
            .confirm(Confirmation::One)
            .freshness(freshness(w, "mise"))
            .launch(Launch::Snapshot {
                snapshot: "mise-outdated".into(),
            })
            .reason(
                Signal::Context,
                &format!("{} tools have newer versions", outdated.len()),
            ),
        );
    }
    let missing = w.mise.missing();
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(|t| t.name.as_str()).collect();
        out.push(
            Item::new(
                "mise.install",
                &format!("Install missing tools · {}", names.join(", ")),
                Kind::Mise,
                ResultType::Recommendation,
                tag_for_dir(
                    w,
                    w.location
                        .project
                        .as_ref()
                        .map(|p| p.root.as_str())
                        .unwrap_or(&w.location.cwd),
                ),
            )
            .summary("install the tools this configuration requests but the host lacks")
            .commands(&[
                "mise install --dry-run",
                "mise install --include-task-tools",
            ])
            .keywords(&["mise", "install", "missing", "tools"])
            .group("Tasks")
            .risk(Risk::Mutating)
            .confirm(Confirmation::One)
            .launch(Launch::Activity {
                script: "generic:mise install --include-task-tools".into(),
            })
            .reason(
                Signal::Urgency,
                &crate::screens::plural(names.len(), "tool missing", "tools missing"),
            ),
        );
    }
    out.push(
        Item::new("mise.status", "Show mise configuration", Kind::Mise, ResultType::Resource, ScopeTag::here(&w.location.cwd))
            .summary("effective configuration from this folder through its ancestors: tools, tasks, environment, trust")
            .commands(&["mise ls --json", "mise tasks ls --all --json", "mise trust --show"])
            .keywords(&["mise", "config", "tools", "trust", "env"])
            .group("Tasks")
            .freshness(freshness(w, "mise"))
            .launch(Launch::Snapshot { snapshot: "mise".into() })
            .reason(Signal::Default, &format!("{} configuration files in effect", w.mise.configs.len())),
    );
}

fn git_items(w: &World, out: &mut Vec<Item>) {
    if let Some(g) = w.git_here() {
        git_here_items(w, g, out);
    }
    git_children_items(w, out);
    github_items(w, out);
}

fn git_here_items(w: &World, g: &crate::domain::stack::GitState, out: &mut Vec<Item>) {
    let tag = tag_for_dir(w, &g.path);
    let root_short = w.location.short(&g.path);
    let fresh = freshness(w, "git");
    let branch = g
        .branch
        .clone()
        .unwrap_or_else(|| format!("detached @ {}", g.head_short));
    out.push(
        Item::new(
            "git.status",
            "Show git status",
            Kind::Git,
            ResultType::Resource,
            tag.clone(),
        )
        .summary(&format!(
            "worktree, branch, upstream and stash state of {root_short}"
        ))
        .commands(&[&g.status_cmd()])
        .keywords(&["git", "status", "branch", "worktree", "changes"])
        .group("Git")
        .freshness(fresh.clone())
        .launch(Launch::Snapshot {
            snapshot: "git".into(),
        })
        .reason(Signal::Context, &format!("on {branch}")),
    );
    if g.dirty() {
        let n = g.changed() + g.untracked;
        out.push(
            Item::new(
                "git.review",
                &format!("Review {n} modified files"),
                Kind::Git,
                ResultType::Recommendation,
                tag.clone(),
            )
            .summary("show the changed files with their diff stat before committing or syncing")
            .commands(&[
                &format!("git -C {} status --short", g.path),
                &format!("git -C {} diff --stat", g.path),
            ])
            .keywords(&["git", "diff", "changes", "modified", "review", "dirty"])
            .group("Git")
            .freshness(fresh.clone())
            .launch(Launch::Snapshot {
                snapshot: "git-diff".into(),
            })
            .alt("Open in a Git TUI (lazygit)", "git.tui")
            .reason(
                Signal::Urgency,
                &format!("{} modified · {} untracked", g.changed(), g.untracked),
            ),
        );
    }
    let mut pull = Item::new(
        "git.pull",
        "Pull",
        Kind::Git,
        ResultType::Action,
        tag.clone(),
    )
    .summary(&format!(
        "fast-forward {root_short} to {}",
        g.upstream.clone().unwrap_or("its upstream".into())
    ))
    .commands(&[&format!("git -C {} pull --ff-only", g.path)])
    .keywords(&["git", "pull", "sync", "fetch", "update", "behind"])
    .group("Git")
    .risk(Risk::Mutating)
    .freshness(fresh.clone())
    .launch(Launch::Activity {
        script: "git:pull".into(),
    })
    .alt("Fetch only", "git.fetch")
    .alt("Push", "git.push");
    if g.behind > 0 && !g.diverged {
        pull = pull
            .reason(
                Signal::Urgency,
                &format!("branch is {} commits behind", g.behind),
            )
            .effects(&[&format!(
                "{} commits arrive · nothing is rewritten",
                g.behind
            )]);
    } else if let Some(r) = g.block_reason() {
        pull = pull
            .effects(&[&format!(
                "blocked · {r} · nothing runs until it is resolved"
            )])
            .confirm(Confirmation::One)
            .reason(Signal::Context, &format!("blocked · {r}"));
    } else {
        pull = pull
            .effects(&["already up to date"])
            .reason(Signal::Default, "up to date");
    }
    out.push(pull);
    if g.ahead > 0 {
        out.push(
            Item::new(
                "git.push",
                "Push",
                Kind::Git,
                ResultType::Action,
                tag.clone(),
            )
            .summary(&format!(
                "push {} commits of {branch} to {}",
                g.ahead,
                g.upstream.clone().unwrap_or("origin".into())
            ))
            .commands(&[
                &format!("git -C {} push --dry-run", g.path),
                &format!("git -C {} push", g.path),
            ])
            .keywords(&["git", "push", "upload", "ahead"])
            .group("Git")
            .risk(Risk::Mutating)
            .confirm(Confirmation::One)
            .freshness(fresh.clone())
            .launch(Launch::Activity {
                script: format!("generic:git -C {} push", g.path),
            })
            .reason(Signal::Context, &format!("{} commits ahead", g.ahead)),
        );
    }
    if g.branch.as_deref() != Some(g.primary.as_str()) {
        let mut sw = Item::new(
            "git.switch_primary",
            &format!("Switch to {} (primary branch)", g.primary),
            Kind::Git,
            ResultType::Action,
            tag.clone(),
        )
        .summary(&format!(
            "switch {root_short} to its resolved primary branch {}",
            g.primary
        ))
        .commands(&[&format!("git -C {} switch {}", g.path, g.primary)])
        .keywords(&[
            "git",
            "checkout main",
            "checkout master",
            "switch",
            "primary",
            "default branch",
        ])
        .group("Git")
        .risk(Risk::Mutating)
        .freshness(fresh.clone())
        .launch(Launch::Activity {
            script: format!("generic:git -C {} switch {}", g.path, g.primary),
        })
        .reason(
            Signal::Context,
            &format!("primary branch resolves to {}", g.primary),
        );
        if let Some(r) = g.block_reason() {
            sw = sw
                .effects(&[&format!("blocked · {r} · work is never discarded")])
                .confirm(Confirmation::One);
        }
        out.push(sw);
    }
    if g.in_progress.is_some() || g.conflicts > 0 {
        out.push(
            Item::new("git.tui", "Resolve the rebase in lazygit", Kind::Git, ResultType::Handoff, tag.clone())
                .summary("conflict and history work goes to the specialist Git TUI; holla returns when it exits")
                .commands(&[&format!("lazygit -p {}", g.path)])
                .keywords(&["git", "rebase", "conflicts", "lazygit", "resolve", "merge"])
                .group("Git")
                .freshness(fresh.clone())
                .launch(Launch::Handoff { tool: "lazygit".into() })
                .reason(Signal::Urgency, &format!("{} in progress · {} conflicts", g.in_progress.clone().unwrap_or("operation".into()), g.conflicts)),
        );
    } else {
        out.push(
            Item::new(
                "git.tui",
                "Open lazygit",
                Kind::Git,
                ResultType::Handoff,
                tag.clone(),
            )
            .summary("history, staging and rebasing in the specialist Git TUI")
            .commands(&[&format!("lazygit -p {}", g.path)])
            .keywords(&["git", "lazygit", "history", "stage", "commit"])
            .group("Git")
            .launch(Launch::Handoff {
                tool: "lazygit".into(),
            })
            .reason(Signal::Default, "installed on this host"),
        );
    }
    if g.stash > 0 {
        out.push(
            Item::new(
                "git.stash",
                &format!("Show stash ({})", g.stash),
                Kind::Git,
                ResultType::Resource,
                tag.clone(),
            )
            .summary("list stashed changes")
            .commands(&[&format!("git -C {} stash list", g.path)])
            .keywords(&["git", "stash"])
            .group("Git")
            .launch(Launch::Snapshot {
                snapshot: "git-stash".into(),
            })
            .reason(Signal::Context, &format!("{} stash entries", g.stash)),
        );
    }
}

fn git_children_items(w: &World, out: &mut Vec<Item>) {
    let here = w.git_here().map(|g| g.path.clone());
    let children: Vec<&crate::domain::stack::GitState> = w
        .git
        .iter()
        .filter(|x| Some(&x.path) != here.as_ref())
        .collect();
    if !children.is_empty() {
        let dirty = children.iter().filter(|x| x.dirty()).count();
        let behind = children.iter().filter(|x| x.behind > 0).count();
        let n = children.iter().filter(|x| x.worktree_of.is_none()).count();
        let ctag = ScopeTag::child(&w.location.cwd);
        out.push(
            Item::new("git.status_all", &format!("Status across {n} child projects"), Kind::Git, ResultType::Recommendation, ctag.clone())
                .summary("inspect every discovered Git project in parallel · worktrees deduplicated, submodules marked")
                .commands(&["git -C <each> status --porcelain=v2 --branch --show-stash"])
                .keywords(&["git", "status", "children", "projects", "all", "collection", "workspace"])
                .group("Git")
                .freshness(freshness(w, "children"))
                .launch(Launch::Snapshot { snapshot: "git-all".into() })
                .reason(Signal::Urgency, &format!("{dirty} dirty · {behind} behind")),
        );
        out.push(
            Item::new("git.pull_all", "Pull every child project", Kind::Plan, ResultType::Flow, ctag.clone())
                .summary("fast-forward-only pulls across the eligible projects, as a reviewable plan with exclusions")
                .commands(&["git -C <each> pull --ff-only"])
                .keywords(&["git", "pull", "sync projects", "all", "children", "synchronize"])
                .group("Git")
                .risk(Risk::Mutating)
                .confirm(Confirmation::One)
                .launch(Launch::Plan { plan: "git-pull-all".into() })
                .reason(Signal::Context, &format!("{behind} projects behind · {dirty} blocked by local changes")),
        );
        out.push(
            Item::new("git.switch_all", "Switch every project to its primary branch", Kind::Plan, ResultType::Flow, ctag)
                .summary("resolve each project's primary branch (main, master, develop…) and switch the eligible ones")
                .commands(&["git -C <each> switch <primary>"])
                .keywords(&["git", "checkout main", "checkout master", "switch to default", "switch to primary branch", "children"])
                .group("Git")
                .risk(Risk::Mutating)
                .confirm(Confirmation::One)
                .launch(Launch::Plan { plan: "git-switch-primary".into() })
                .reason(Signal::Default, "primary branch resolved per project, never hard-coded"),
        );
    }
}

fn github_items(w: &World, out: &mut Vec<Item>) {
    if w.github.logged_in {
        out.push(
            Item::new("github.clone", "Clone a GitHub repository…", Kind::Git, ResultType::Action, ScopeTag::here(&w.location.cwd))
                .summary(&format!("choose a repository from {} or its organizations, review owner, protocol, destination and primary branch, then clone", w.github.account))
                .commands(&["gh repo clone <owner>/<repository> <destination>", "gh auth status --active", "gh org list --limit 100", "gh repo list <owner> --limit 100 --no-archived"])
                .keywords(&["clone", "github", "gh", "repository", "checkout"])
                .group("Git")
                .risk(Risk::Mutating)
                .confirm(Confirmation::One)
                .freshness(freshness(w, "github"))
                .args(vec![
                    ArgSpec::choice("owner", &{
                        let mut v = vec![w.github.account.as_str()];
                        v.extend(w.github.orgs.iter().map(|s| s.as_str()));
                        v
                    }, 0),
                    ArgSpec::text("repository", "name", "").required().help("searched in the owner's repositories"),
                    ArgSpec::choice("protocol", &["ssh", "https"], 0),
                    ArgSpec::text("destination", "folder", &w.location.cwd_short()).help("never overwrites an existing folder"),
                ])
                .launch(Launch::Activity { script: "generic:gh repo clone".into() })
                .reason(Signal::Default, &format!("logged in as {}", w.github.account)),
        );
    } else if let SourceState::Failed(r) = w.source_state("github") {
        out.push(
            Item::new(
                "github.clone",
                "Clone a GitHub repository…",
                Kind::Git,
                ResultType::Action,
                ScopeTag::here(&w.location.cwd),
            )
            .summary("cloning needs an active gh login")
            .commands(&["gh auth login"])
            .keywords(&["clone", "github", "gh"])
            .group("Git")
            .freshness(Freshness::Unavailable(r))
            .launch(Launch::Insert)
            .reason(Signal::Default, "gh is not logged in"),
        );
    }
}

fn rust_items(w: &World, out: &mut Vec<Item>) {
    let Some(p) = &w.location.project else {
        return;
    };
    let crate::domain::context::ProjectKind::Rust { workspace, member } = &p.kind else {
        return;
    };
    let tag = tag_for_dir(w, &p.root);
    let ws = if *workspace { " --workspace" } else { "" };
    let fresh = freshness(w, "cargo");
    let mut push = |id: &str, label: &str, cmd: &str, summary: &str, kw: &[&str], risk: Risk| {
        out.push(
            Item::new(id, label, Kind::Rust, ResultType::Action, tag.clone())
                .summary(summary)
                .commands(&[cmd])
                .keywords(kw)
                .group("Tasks")
                .risk(risk)
                .freshness(fresh.clone())
                .launch(Launch::Activity {
                    script: format!("generic:{cmd}"),
                })
                .reason(Signal::Context, "Cargo.toml here"),
        );
    };
    push(
        "cargo.check",
        "cargo check",
        &format!("cargo check{ws}"),
        "type-check every target without producing binaries",
        &["cargo", "check", "compile", "build"],
        Risk::ReadOnly,
    );
    push(
        "cargo.build",
        "cargo build",
        &format!("cargo build{ws}"),
        "build the debug profile",
        &["cargo", "build", "compile"],
        Risk::ReadOnly,
    );
    push(
        "cargo.test",
        "cargo test",
        &format!("cargo test{ws}"),
        "run the tests with the built-in harness",
        &["cargo", "test", "tests"],
        Risk::ReadOnly,
    );
    push(
        "cargo.clippy",
        "cargo clippy",
        &format!("cargo clippy{ws} --all-targets -- -D warnings"),
        "lint every target with warnings denied",
        &["cargo", "clippy", "lint"],
        Risk::ReadOnly,
    );
    push(
        "cargo.fmt",
        "cargo fmt --check",
        "cargo fmt --all -- --check",
        "check formatting",
        &["cargo", "fmt", "format"],
        Risk::ReadOnly,
    );
    if let Some(m) = member {
        push(
            "cargo.run",
            &format!("cargo run -p {m}"),
            &format!("cargo run -p {m}"),
            "run the member this folder belongs to",
            &["cargo", "run"],
            Risk::ReadOnly,
        );
    }
    if let Some(c) = w
        .disk
        .candidates
        .iter()
        .find(|c| c.path == format!("{}/target", p.root))
    {
        out.push(
            Item::new("cargo.clean", &format!("cargo clean · {:.1} GB", c.gb), Kind::Cleanup, ResultType::Action, tag.clone())
                .summary(&format!("remove the resolved target directory {} · {} items · regenerated by the next build", w.location.short(&c.path), c.items))
                .commands(&["cargo clean --dry-run --verbose", "cargo clean"])
                .effects(&[&format!("{:.1} GB freed · next build recompiles everything", c.gb)])
                .keywords(&["cargo", "clean", "target", "artifacts", "space"])
                .group("Disk")
                .risk(Risk::Destructive)
                .confirm(Confirmation::One)
                .launch(Launch::Activity { script: "generic:cargo clean".into() })
                .reason(Signal::Context, &format!("{:.1} GB generated artifacts", c.gb)),
        );
    }
}

fn docker_items(w: &World, out: &mut Vec<Item>) {
    let d = &w.docker;
    let htag = host_tag(w);
    let fresh = freshness(w, "docker");
    if let Err(reason) = &d.daemon {
        // no docker at all is an absent capability, not a failed one
        if reason.contains("not installed") {
            return;
        }
        out.push(
            Item::new("docker.daemon", "Docker", Kind::Docker, ResultType::Resource, htag)
                .summary("the Docker daemon could not be reached; container actions are unavailable until it is")
                .commands(&["docker info"])
                .keywords(&["docker", "containers", "daemon"])
                .group("Services")
                .freshness(Freshness::Unavailable(reason.clone()))
                .launch(Launch::Snapshot { snapshot: "docker-unavailable".into() })
                .reason(Signal::Default, "capability present, daemon unreachable"),
        );
        return;
    }
    out.push(
        Item::new(
            "docker.ps",
            "List containers",
            Kind::Docker,
            ResultType::Resource,
            htag.clone(),
        )
        .summary(&format!(
            "{} containers · {} running · with image, state, ports, project and size",
            d.containers.len(),
            d.running()
        ))
        .commands(&["docker ps -a --size"])
        .keywords(&["docker", "containers", "ps", "list", "running"])
        .group("Services")
        .freshness(fresh.clone())
        .launch(Launch::Snapshot {
            snapshot: "docker-ps".into(),
        })
        .reason(Signal::Context, &format!("{} running", d.running())),
    );
    out.push(
        Item::new(
            "docker.df",
            "Show Docker disk usage",
            Kind::Docker,
            ResultType::Resource,
            htag.clone(),
        )
        .summary("images, containers, volumes and build cache with reclaimable space")
        .commands(&["docker system df", "docker system df -v"])
        .keywords(&["docker", "disk", "space", "usage", "df", "reclaimable"])
        .group("Services")
        .freshness(fresh.clone())
        .launch(Launch::Snapshot {
            snapshot: "docker-df".into(),
        })
        .reason(
            Signal::Context,
            &format!("{:.1} GB reclaimable", d.reclaimable_gb),
        ),
    );
    for c in &d.containers {
        let scope = match (&c.project, &d.compose) {
            (Some(p), Some(comp)) if p == &comp.name => tag_for_dir(w, &comp.dir),
            _ => htag.clone(),
        };
        let mut item = Item::new(
            &format!("docker.logs.{}", c.name),
            &format!("Follow {} logs", c.name),
            Kind::Docker,
            ResultType::Action,
            scope,
        )
        .summary(&format!(
            "stream the last 10 minutes and follow · {} · {}",
            c.image,
            if c.running { "running" } else { "exited" }
        ))
        .commands(&[&format!("docker logs -f --since 10m {}", c.name)])
        .keywords(&[
            "docker",
            "logs",
            "service logs",
            &c.name,
            "follow",
            "container",
        ])
        .group("Services")
        .freshness(fresh.clone())
        .launch(Launch::Activity {
            script: if c.name == "acme-api-1" {
                "docker:logs-api".into()
            } else {
                format!("generic:docker logs -f --since 10m {}", c.name)
            },
        })
        .alt("Restart", &format!("docker.restart.{}", c.name))
        .alt("Stop", &format!("docker.stop.{}", c.name));
        item = match c.health.as_deref() {
            Some("unhealthy") => item.reason(Signal::Urgency, "service is unhealthy"),
            _ if c.running => item.reason(Signal::Context, "running"),
            _ => item.reason(Signal::Default, "exited"),
        };
        out.push(item);
    }
    if let Some(comp) = &d.compose {
        let ctag = tag_for_dir(w, &comp.dir);
        let app_services: Vec<&str> = comp
            .services
            .iter()
            .filter(|s| !matches!(s.as_str(), "db" | "redis"))
            .map(|s| s.as_str())
            .collect();
        out.push(
            Item::new("docker.compose_logs", &format!("Follow logs from {}", app_services.join(", ")), Kind::Docker, ResultType::Action, ctag.clone())
                .summary(&format!("project-aware multi-service logs of Compose project {} · each stream keeps its identity", comp.name))
                .commands(&[&format!("docker compose logs -f --tail 200 {}", app_services.join(" "))])
                .keywords(&["docker", "compose", "logs", "service logs", "follow", "api", "worker", "scheduler"])
                .group("Services")
                .freshness(fresh.clone())
                .launch(Launch::Activity { script: "logs:acme".into() })
                .reason(Signal::Context, &format!("Compose project {} defined here", comp.name)),
        );
        out.push(
            Item::new(
                "docker.compose_ps",
                "Show Compose services",
                Kind::Docker,
                ResultType::Resource,
                ctag.clone(),
            )
            .summary(&format!("services of {} with state and health", comp.name))
            .commands(&["docker compose ps --all"])
            .keywords(&["docker", "compose", "services", "health"])
            .group("Services")
            .freshness(fresh.clone())
            .launch(Launch::Snapshot {
                snapshot: "compose-ps".into(),
            })
            .reason(
                Signal::Context,
                &format!("{} unhealthy", d.unhealthy().len()),
            ),
        );
        out.push(
            Item::new(
                "docker.compose_down",
                "Stop the Compose project",
                Kind::Docker,
                ResultType::Action,
                ctag,
            )
            .summary(&format!(
                "stop and remove the containers of {} · volumes are kept",
                comp.name
            ))
            .commands(&["docker compose stop", "docker compose down"])
            .keywords(&["docker", "compose", "down", "stop"])
            .group("Services")
            .risk(Risk::Mutating)
            .confirm(Confirmation::One)
            .launch(Launch::Activity {
                script: "generic:docker compose down".into(),
            })
            .reason(Signal::Default, "project containers only"),
        );
    }
    let running: Vec<&str> = d
        .containers
        .iter()
        .filter(|c| c.running)
        .map(|c| c.name.as_str())
        .collect();
    out.push(
        Item::new(
            "docker.stop_all",
            "Stop all containers",
            Kind::Docker,
            ResultType::Action,
            htag.clone(),
        )
        .summary(&format!(
            "stop the {} running containers on {} gracefully · nothing is removed",
            running.len(),
            w.host.name
        ))
        .commands(&[&format!("docker stop {}", running.join(" "))])
        .effects(&[&format!(
            "{} containers stop · data and images stay",
            running.len()
        )])
        .keywords(&["docker", "stop", "all", "containers", "clean", "cleanup"])
        .group("Services")
        .risk(Risk::Mutating)
        .confirm(Confirmation::One)
        .freshness(fresh.clone())
        .launch(Launch::Activity {
            script: format!("generic:docker stop {}", running.join(" ")),
        })
        .reason(
            Signal::Context,
            &format!("{} running on {}", running.len(), w.host.name),
        ),
    );
    let all: Vec<&str> = d.containers.iter().map(|c| c.name.as_str()).collect();
    out.push(
        Item::new("docker.remove_all", "Stop and remove all containers", Kind::Docker, ResultType::Action, htag.clone())
            .summary(&format!("stop the running containers, then remove every container on {} · images and volumes stay", w.host.name))
            .commands(&[&format!("docker stop {}", running.join(" ")), &format!("docker rm {}", all.join(" ")), "docker ps -a"])
            .effects(&[&format!("{} containers removed · writable layers lost", all.len())])
            .keywords(&["docker", "remove", "rm", "all", "containers", "reset", "clean", "cleanup"])
            .group("Services")
            .risk(Risk::Destructive)
            .confirm(Confirmation::TwoGate { phrase: format!("REMOVE ALL CONTAINERS ON {}", w.host.name), broad: false })
            .freshness(fresh.clone())
            .launch(Launch::Activity { script: format!("generic:docker rm {}", all.join(" ")) })
            .reason(Signal::Default, &format!("{} containers on {}", all.len(), w.host.name)),
    );
    out.push(
        Item::new(
            "docker.images_prune",
            "Remove all unused images",
            Kind::Docker,
            ResultType::Action,
            htag.clone(),
        )
        .summary(&format!(
            "remove every image not used by a container · {} images · {:.1} GB",
            d.images, d.images_gb
        ))
        .commands(&["docker image prune -a"])
        .effects(&[&format!(
            "{:.1} GB freed · images re-pull on next use",
            d.images_gb
        )])
        .keywords(&[
            "docker", "images", "prune", "remove", "rmi", "clean", "cleanup",
        ])
        .group("Services")
        .risk(Risk::Destructive)
        .confirm(Confirmation::TwoGate {
            phrase: format!("REMOVE ALL IMAGES ON {}", w.host.name),
            broad: false,
        })
        .launch(Launch::Activity {
            script: "generic:docker image prune -a".into(),
        })
        .reason(Signal::Default, &format!("{} dangling", d.dangling_images)),
    );
    out.push(
        Item::new(
            "docker.network_prune",
            "Prune unused networks",
            Kind::Docker,
            ResultType::Action,
            htag.clone(),
        )
        .summary(&format!(
            "remove custom networks no container uses · {}",
            d.networks
        ))
        .commands(&["docker network prune"])
        .keywords(&["docker", "network", "prune", "clean", "cleanup"])
        .group("Services")
        .risk(Risk::Mutating)
        .confirm(Confirmation::One)
        .launch(Launch::Activity {
            script: "generic:docker network prune".into(),
        })
        .reason(Signal::Default, "available on this host"),
    );
    out.push(
        Item::new("docker.volume_prune", "Prune unused volumes", Kind::Docker, ResultType::Action, htag.clone())
            .summary(&format!("remove every volume no container uses · {} named · {:.1} GB · named volumes hold durable data", d.named_volumes(), d.volumes_gb()))
            .commands(&["docker volume prune -a"])
            .effects(&[&format!("{:.1} GB freed · permanent", d.volumes_gb())])
            .keywords(&["docker", "volumes", "prune", "data", "clean", "cleanup"])
            .group("Services")
            .risk(Risk::Destructive)
            .confirm(Confirmation::TwoGate { phrase: format!("PRUNE ALL VOLUMES ON {}", w.host.name), broad: false })
            .launch(Launch::Activity { script: "generic:docker volume prune -a".into() })
            .reason(Signal::Default, &format!("{} volumes", d.volumes.len())),
    );
    out.push(
        Item::new(
            "docker.builder_prune",
            "Prune builder cache",
            Kind::Docker,
            ResultType::Action,
            htag.clone(),
        )
        .summary(&format!(
            "remove build cache · {:.1} GB · rebuilt on the next build",
            d.builder_cache_gb
        ))
        .commands(&["docker builder prune -a", "docker buildx prune -a"])
        .keywords(&[
            "docker", "builder", "buildx", "cache", "prune", "clean", "cleanup",
        ])
        .group("Services")
        .risk(Risk::Mutating)
        .confirm(Confirmation::One)
        .launch(Launch::Activity {
            script: "generic:docker builder prune -a".into(),
        })
        .reason(
            Signal::Context,
            &format!("{:.1} GB of build cache", d.builder_cache_gb),
        ),
    );
    out.push(
        Item::new("docker.cleanup", "Clean Docker completely", Kind::Plan, ResultType::Flow, htag)
            .summary(&format!("remove all user-removable Docker state on {}: containers, images, networks, volumes, system data, build cache · as a reviewable plan", w.host.name))
            .commands(&[&format!("docker stop {}", running.join(" ")), &format!("docker rm {}", all.join(" ")), "docker image prune -a --force", "docker network prune --force", "docker volume prune -a --force", "docker builder prune -a --force", "docker buildx prune -a --force", "docker system df"])
            .effects(&[&format!("{} containers · {} images · {} volumes · ~{:.1} GB reclaimed · permanent", d.containers.len(), d.images, d.volumes.len(), d.reclaimable_gb)])
            .keywords(&["docker", "docker cleanup", "docker clean", "clean", "prune", "everything", "reset", "remove all", "complete"])
            .group("Services")
            .risk(Risk::Destructive)
            .confirm(Confirmation::TwoGate { phrase: format!("REMOVE ALL DOCKER DATA ON {}", w.host.name), broad: true })
            .freshness(fresh)
            .launch(Launch::Plan { plan: "docker-cleanup".into() })
            .alt("Stop all containers only", "docker.stop_all")
            .alt("Prune builder cache only", "docker.builder_prune")
            .reason(Signal::Context, &format!("{:.1} GB reclaimable on {}", d.reclaimable_gb, w.host.name)),
    );
}

fn system_items(w: &World, out: &mut Vec<Item>) {
    let s = &w.system;
    let htag = host_tag(w);
    let fresh = freshness(w, "system");
    let mut res = Item::new("system.resources", "Show system resources", Kind::System, ResultType::Resource, htag.clone())
        .summary("CPU, load, memory, swap, pressure, disk and network snapshot; then hand off to btm for continuous monitoring")
        .commands(&["holla snapshot system", "btm"])
        .keywords(&["system", "resources", "cpu", "memory", "load", "pressure", "what uses cpu", "what uses memory", "slow"])
        .group("System")
        .freshness(fresh.clone())
        .launch(Launch::Snapshot { snapshot: "system".into() })
        .alt("Open btm", "system.monitor")
        .alt("Show process tree", "system.processes");
    res = if s.pressure_minutes > 0 {
        res.reason(
            Signal::Urgency,
            &format!("CPU {}% for {} min", s.cpu_pct, s.pressure_minutes),
        )
    } else {
        res.reason(
            Signal::Context,
            &format!(
                "CPU {}% · memory {:.0}%",
                s.cpu_pct,
                s.mem_used_gb / s.mem_total_gb * 100.0
            ),
        )
    };
    out.push(res);
    if s.btm_installed {
        out.push(
            Item::new("system.monitor", "Open btm", Kind::System, ResultType::Handoff, htag.clone())
                .summary("persistent deep monitoring in the specialist tool; the activity keeps running while you return to holla")
                .commands(&["btm"])
                .keywords(&["btm", "bottom", "monitor", "top", "htop", "processes", "system monitor"])
                .group("System")
                .freshness(fresh.clone())
                .launch(Launch::Handoff { tool: "btm".into() })
                .reason(if s.pressure_minutes > 0 { Signal::Urgency } else { Signal::Default }, if s.pressure_minutes > 0 { "sustained pressure · continuous view fits" } else { "installed on this host" }),
        );
    }
    out.push(
        Item::new(
            "system.processes",
            "Show process tree",
            Kind::System,
            ResultType::Resource,
            htag.clone(),
        )
        .summary("processes by CPU and memory with their hierarchy")
        .commands(&["ps -eo pid,ppid,pcpu,rss,stat,comm --forest"])
        .keywords(&["process", "processes", "tree", "ps", "pid"])
        .group("System")
        .freshness(fresh.clone())
        .launch(Launch::Snapshot {
            snapshot: "processes".into(),
        })
        .reason(Signal::Default, &format!("{} noted processes", s.top.len())),
    );
    out.push(
        Item::new(
            "system.port",
            "Find the process on a port…",
            Kind::System,
            ResultType::Action,
            htag.clone(),
        )
        .summary("which process listens on a port, with its PID and parent")
        .commands(&["lsof -nP -iTCP:<port> -sTCP:LISTEN"])
        .keywords(&[
            "port",
            "listen",
            "eaddrinuse",
            "address already in use",
            "lsof",
            "process on port",
        ])
        .group("System")
        .args(vec![ArgSpec::text("port", "TCP port", "5173").required()])
        .launch(Launch::Snapshot {
            snapshot: "port".into(),
        })
        .reason(Signal::Default, "available on this host"),
    );
    out.push(
        Item::new("system.kill", "Stop a process…", Kind::System, ResultType::Action, htag.clone())
            .summary("send a signal to one process after reviewing its identity, hierarchy and likely effect")
            .commands(&["kill -TERM <pid>"])
            .keywords(&["kill", "stop process", "signal", "terminate"])
            .group("System")
            .risk(Risk::Destructive)
            .confirm(Confirmation::One)
            .args(vec![ArgSpec::text("pid", "process id", "").required(), ArgSpec::choice("signal", &["TERM", "INT", "KILL"], 0)])
            .launch(Launch::Activity { script: "generic:kill -TERM".into() })
            .reason(Signal::Default, "reviewed before sending"),
    );
    if w.apt.is_some() {
        let n = w.apt.as_ref().map(|a| a.upgradable.len()).unwrap_or(0);
        out.push(
            Item::new("system.upgrade", "Upgrade everything on this system", Kind::Plan, ResultType::Flow, htag.clone())
                .summary(&format!("Debian packages and global mise tools on {} as a dependency graph: preflight, parallel discovery, review, apply, optional cleanup, verification", w.host.name))
                .commands(&["sudo apt-get update", "apt list --upgradable", "sudo apt-get upgrade -y", "mise outdated --json", "mise upgrade", "sudo apt-get autoremove --purge -y"])
                .effects(&[&format!("{n} Debian packages · 4 mise tools · reboot required")])
                .keywords(&["upgrade", "update", "everything", "apt", "packages", "system", "debian", "mise"])
                .group("System")
                .risk(Risk::Privileged)
                .confirm(Confirmation::One)
                .freshness(freshness(w, "apt"))
                .launch(Launch::Plan { plan: "upgrade".into() })
                .reason(Signal::Urgency, &format!("{n} upgradable · 2 security")),
        );
    }
}

fn disk_items(w: &World, out: &mut Vec<Item>) {
    let d = &w.disk;
    let here = ScopeTag::here(&w.location.cwd);
    let htag = host_tag(w);
    let fresh = freshness(w, "filesystem");
    let mut usage = Item::new("disk.usage", "Analyze disk usage", Kind::Disk, ResultType::Flow, here.clone())
        .summary("largest-first view of this folder that deepens as the scan streams; select rebuildable artifacts for cleanup")
        .commands(&[&format!("holla disk analyze {}", w.location.cwd)])
        .keywords(&["disk", "usage", "why disk full", "space", "largest", "du", "analyze", "full"])
        .aliases(&["du"])
        .page_aliases(&["u"])
        .group("Disk")
        .freshness(fresh.clone())
        .launch(Launch::Disk { path: w.location.cwd.clone() })
        .alt("Analyze the home folder", "disk.usage_home")
        .alt("Analyze a volume", "disk.usage_volume");
    if let Some(fs) = d.root_fs()
        && fs.pct() >= 90
    {
        usage = usage.reason(
            Signal::Urgency,
            &format!("{} is {}% full", fs.mount, fs.pct()),
        );
    } else if let Some(fs) = d.root_fs() {
        usage = usage.reason(
            Signal::Default,
            &format!("{} is {}% full", fs.mount, fs.pct()),
        );
    }
    out.push(usage);
    out.push(
        Item::new(
            "disk.usage_home",
            "Analyze the home folder",
            Kind::Disk,
            ResultType::Flow,
            ScopeTag::host(&w.host.name),
        )
        .summary("largest-first view of the home directory")
        .commands(&[&format!("holla disk analyze {}", w.location.home)])
        .keywords(&["disk", "home", "usage"])
        .page_aliases(&["h"])
        .group("Disk")
        .launch(Launch::Disk {
            path: w.location.home.clone(),
        })
        .reason(Signal::Default, "available"),
    );
    let local: Vec<&crate::domain::stack::Candidate> = d
        .candidates
        .iter()
        .filter(|c| c.path.starts_with(&w.location.cwd))
        .collect();
    if !local.is_empty() {
        let total: f32 = local.iter().map(|c| c.gb).sum();
        let stale = local.iter().filter(|c| c.default_selected()).count();
        out.push(
            Item::new(
                "disk.artifacts",
                &format!("Inspect {:.0} GB of generated artifacts", total),
                Kind::Disk,
                ResultType::Recommendation,
                here.clone(),
            )
            .summary(&format!(
                "{} below this folder · {} stale enough to start selected",
                crate::screens::plural(
                    local.len(),
                    "rebuildable candidate",
                    "rebuildable candidates"
                ),
                stale
            ))
            .commands(&[&format!(
                "holla disk analyze {} --artifacts",
                w.location.cwd
            )])
            .keywords(&[
                "artifacts",
                "target",
                "node_modules",
                "build",
                "generated",
                "rebuildable",
                "clean",
            ])
            .group("Disk")
            .freshness(fresh.clone())
            .launch(Launch::Disk {
                path: w.location.cwd.clone(),
            })
            .reason(
                Signal::Urgency,
                &format!("{:.0} GB generated artifacts", total),
            ),
        );
        if local.len() > 1 {
            out.push(
                Item::new("disk.clean_children", &format!("Clean developer artifacts under {}", w.location.cwd_short()), Kind::Plan, ResultType::Flow, ScopeTag::child(&w.location.cwd))
                    .summary("Cargo targets, Gradle outputs and Node dependency trees beneath their owning projects as a plan · shared targets serialised, independent projects in parallel")
                    .commands(&["cargo clean", "./gradlew clean", "trash <node_modules>", "pnpm store prune"])
                    .effects(&[&format!("~{:.1} GB reclaimable · recent and unverifiable data stays unselected", d.reclaimable_gb())])
                    .keywords(&["clean", "artifacts", "children", "rust", "gradle", "node", "cleanup", "build"])
                    .group("Disk")
                    .risk(Risk::Destructive)
                    .confirm(Confirmation::TwoGate { phrase: format!("REMOVE ALL BUILD ARTIFACTS UNDER {}", w.location.cwd), broad: false })
                    .launch(Launch::Plan { plan: "cleanup-work".into() })
                    .reason(Signal::Context, &format!("{} candidates in {} projects", local.len(), w.location.children.len())),
            );
        }
    }
    if !d.history.is_empty() {
        out.push(
            Item::new(
                "disk.history",
                "Show cleanup history",
                Kind::Disk,
                ResultType::Resource,
                htag.clone(),
            )
            .summary("what was removed, how, how much came back, and what was skipped")
            .commands(&["holla disk history"])
            .keywords(&["history", "cleanup", "audit", "reclaimed"])
            .page_aliases(&["y"])
            .group("Disk")
            .launch(Launch::Snapshot {
                snapshot: "disk-history".into(),
            })
            .reason(Signal::Default, &format!("{} entries", d.history.len())),
        );
    }
    // destructive, first-class, always present
    let items = d
        .candidates
        .iter()
        .find(|c| c.path == w.location.cwd)
        .map(|c| (c.items, c.gb));
    out.push(
        Item::new("disk.delete_all", "Delete everything inside this folder", Kind::Cleanup, ResultType::Action, here)
            .summary(&format!("remove every file and folder below {} including hidden and nested content · the folder itself stays", w.location.cwd_short()))
            .commands(&[&format!("find {} -mindepth 1 -delete", w.location.cwd)])
            .effects(&[&match items {
                Some((n, gb)) => format!("{n} items · {gb:.1} GB · permanent"),
                None => "every item below the folder · permanent".to_owned(),
            }])
            .keywords(&["delete", "everything", "empty", "folder", "rm -rf", "wipe", "clear"])
            .group("Disk")
            .risk(Risk::Destructive)
            .confirm(Confirmation::TwoGate { phrase: format!("DELETE EVERYTHING IN {}", w.location.cwd), broad: false })
            .launch(Launch::Activity { script: format!("generic:find {} -mindepth 1 -delete", w.location.cwd) })
            .reason(Signal::Default, "always available · two gates"),
    );
}

fn pg_items(w: &World, out: &mut Vec<Item>) {
    let Some(p) = &w.pg else {
        return;
    };
    let tag = w
        .location
        .project
        .as_ref()
        .map(|pr| tag_for_dir(w, &pr.root))
        .unwrap_or_else(|| host_tag(w));
    let fresh = freshness(w, "postgres");
    let blocked = p.blocked();
    out.push(
        Item::new(
            "pg.activity",
            "Show PostgreSQL activity",
            Kind::Postgres,
            ResultType::Resource,
            tag.clone(),
        )
        .summary(&format!(
            "{} · connections {}/{} · sessions by state, wait event, query and transaction age",
            p.label, p.connections, p.max_connections
        ))
        .commands(&["SELECT pid, state, wait_event, now() - xact_start FROM pg_stat_activity"])
        .keywords(&[
            "postgres",
            "postgresql",
            "database",
            "db",
            "activity",
            "connections",
            "sessions",
            "queries",
        ])
        .group("Services")
        .freshness(fresh.clone())
        .launch(Launch::Snapshot {
            snapshot: "pg".into(),
        })
        .alt("Open pg_activity", "pg.handoff")
        .reason(
            Signal::Context,
            &format!("{}/{} connections", p.connections, p.max_connections),
        ),
    );
    if !blocked.is_empty() {
        let age = blocked.iter().map(|s| s.query_secs).max().unwrap_or(0) / 60;
        out.push(
            Item::new("pg.blocking", "Who is blocking the database?", Kind::Postgres, ResultType::Recommendation, tag.clone())
                .summary("blocker dependency tree with query and transaction ages, wait state, database, user, client and affected sessions")
                .commands(&["SELECT pid, pg_blocking_pids(pid), state, wait_event_type, query FROM pg_stat_activity WHERE cardinality(pg_blocking_pids(pid)) > 0"])
                .keywords(&["blocking", "blocked", "lock", "who is blocking", "postgres", "waiting", "stuck"])
                .group("Services")
                .freshness(fresh.clone())
                .launch(Launch::Snapshot { snapshot: "pg-blocking".into() })
                .alt("Cancel the blocking query", "pg.cancel")
                .alt("Open pg_activity", "pg.handoff")
                .reason(Signal::Urgency, &format!("{} sessions blocked for {age} min", blocked.len())),
        );
        if let Some(b) = p.blockers().first() {
            out.push(
                Item::new("pg.cancel", &format!("Cancel the blocking query (PID {})", b.pid), Kind::Postgres, ResultType::Action, tag.clone())
                    .summary(&format!("pg_cancel_backend({}) · {} · {} · idle in transaction for {} min · revalidated immediately before running", b.pid, b.user, b.app, b.txn_secs / 60))
                    .commands(&[&format!("SELECT pg_cancel_backend({})", b.pid)])
                    .effects(&["the current query is cancelled · the session stays connected"])
                    .keywords(&["cancel", "query", "postgres", "blocking", "pid"])
                    .group("Services")
                    .risk(Risk::Destructive)
                    .confirm(Confirmation::One)
                    .launch(Launch::Activity { script: format!("generic:psql -c 'SELECT pg_cancel_backend({})'", b.pid) })
                    .alt("Terminate the backend instead", "pg.terminate")
                    .reason(Signal::Context, "offered before termination"),
            );
            out.push(
                Item::new(
                    "pg.terminate",
                    &format!("Terminate backend {}", b.pid),
                    Kind::Postgres,
                    ResultType::Action,
                    tag.clone(),
                )
                .summary(&format!(
                    "pg_terminate_backend({}) · disconnects {}@{} · use after cancel fails",
                    b.pid, b.user, b.app
                ))
                .commands(&[&format!("SELECT pg_terminate_backend({})", b.pid)])
                .effects(&["the session is disconnected · its transaction rolls back"])
                .keywords(&["terminate", "kill", "backend", "postgres", "session"])
                .group("Services")
                .risk(Risk::Destructive)
                .confirm(Confirmation::TwoGate {
                    phrase: format!("TERMINATE BACKEND {} ON {}", b.pid, w.host.name),
                    broad: false,
                })
                .launch(Launch::Activity {
                    script: format!("generic:psql -c 'SELECT pg_terminate_backend({})'", b.pid),
                })
                .reason(Signal::Default, "cancel comes first"),
            );
        }
    }
    if p.pg_activity_installed {
        out.push(
            Item::new("pg.handoff", "Open pg_activity", Kind::Postgres, ResultType::Handoff, tag)
                .summary(&format!("live PostgreSQL activity in the specialist tool · {} · password never on the command line", p.label))
                .commands(&[&format!("pg_activity -h {} -p {} -U {} -d {}", p.host, p.port, p.user, p.db)])
                .keywords(&["pg_activity", "postgres", "monitor", "live", "top"])
                .group("Services")
                .freshness(fresh)
                .launch(Launch::Handoff { tool: "pg_activity".into() })
                .reason(if blocked.is_empty() { Signal::Default } else { Signal::Context }, if blocked.is_empty() { "installed on this host" } else { "live view of the blocking tree" }),
        );
    }
}

fn ssh_items(w: &World, out: &mut Vec<Item>) {
    for a in &w.ssh.aliases {
        if a.alias == w.host.name {
            continue;
        }
        let dest = format!("{}@{}:{}", a.user, a.host, a.port);
        let mut item = Item::new(
            &format!("ssh.{}", a.alias),
            &format!("Connect to {}", a.alias),
            Kind::Ssh,
            ResultType::Handoff,
            ScopeTag::host(&w.host.name),
        )
        .summary(&format!(
            "ssh {} · resolves to {dest}{} · {}",
            a.alias,
            a.jump
                .as_ref()
                .map(|j| format!(" via {j}"))
                .unwrap_or_default(),
            a.role
        ))
        .commands(&[&format!("ssh {}", a.alias)])
        .effects(&[&format!("interactive session on {}", a.host)])
        .keywords(&["ssh", "connect", "remote", &a.alias, &a.host, &a.role])
        .group("System")
        .freshness(freshness(w, "ssh"))
        .launch(Launch::Handoff {
            tool: format!("ssh {}", a.alias),
        })
        .alt(
            "Show the resolved configuration",
            &format!("ssh.{}.resolve", a.alias),
        )
        .reason(
            Signal::Default,
            &format!("from {}", w.location.short(&a.from_file)),
        );
        if a.role == "production" {
            item = item.risk(Risk::Privileged).confirm(Confirmation::One);
        }
        out.push(item);
    }
}

fn service_items(w: &World, out: &mut Vec<Item>) {
    if w.host.role != crate::domain::context::HostRole::Production {
        return;
    }
    let tag = ScopeTag::remote(&w.host.name);
    let fresh = freshness(w, "systemd");
    let services = [
        ("payments", "active · 2 h", false),
        (
            "payments-worker",
            "activating (auto-restart) · 3 restarts in 10 min",
            true,
        ),
        ("nginx", "active · 41 d", false),
        ("postgresql", "active · 41 d", false),
    ];
    for (name, state, degraded) in services {
        // a unit holla restarted this session is healed, honestly
        let healed = w.healed_units.iter().any(|u| u == name);
        let degraded = degraded && !healed;
        let state = if healed {
            "active · restarted just now"
        } else {
            state
        };
        let mut st = Item::new(
            &format!("service.status.{name}"),
            &format!("Show {name} status"),
            Kind::System,
            ResultType::Resource,
            tag.clone(),
        )
        .summary(&format!("systemd unit {name}.service · {state}"))
        .commands(&[&format!("systemctl status {name}")])
        .keywords(&["service", "status", "systemd", name, "health"])
        .group("Services")
        .freshness(fresh.clone())
        .launch(Launch::Snapshot {
            snapshot: format!("service-{name}"),
        })
        .alt("Follow the journal", &format!("service.journal.{name}"))
        .alt("Restart", &format!("service.restart.{name}"));
        st = if degraded {
            st.reason(Signal::Urgency, "3 restarts in 10 min")
        } else {
            st.reason(Signal::Context, "active")
        };
        out.push(st);
        out.push(
            Item::new(
                &format!("service.journal.{name}"),
                &format!("Follow {name} journal"),
                Kind::System,
                ResultType::Action,
                tag.clone(),
            )
            .summary(&format!("journalctl -u {name} -f"))
            .commands(&[&format!("journalctl -u {name} -f -n 200")])
            .keywords(&["logs", "journal", "service logs", name, "follow"])
            .group("Services")
            .freshness(fresh.clone())
            .launch(Launch::Activity {
                script: if name == "payments-worker" {
                    "journal:payments-worker".into()
                } else {
                    format!("generic:journalctl -u {name} -f")
                },
            })
            .reason(
                if degraded {
                    Signal::Urgency
                } else {
                    Signal::Default
                },
                if degraded {
                    "unit is restarting"
                } else {
                    "available"
                },
            ),
        );
        out.push(
            Item::new(
                &format!("service.restart.{name}"),
                &format!("Restart {name}"),
                Kind::System,
                ResultType::Action,
                tag.clone(),
            )
            .summary(&format!(
                "systemctl restart {name} on {} · production · in-flight requests are dropped",
                w.host.name
            ))
            .commands(&[
                &format!("sudo systemctl restart {name}"),
                &format!("systemctl status {name}"),
            ])
            .effects(&[&format!(
                "{name} restarts · brief outage on {}",
                w.host.name
            )])
            .keywords(&["restart", "service", name, "bounce"])
            .group("Services")
            .risk(Risk::Privileged)
            .confirm(Confirmation::TwoGate {
                phrase: format!("RESTART {} ON {}", name.to_uppercase(), w.host.name),
                broad: false,
            })
            .launch(Launch::Activity {
                script: format!("generic:sudo systemctl restart {name}"),
            })
            .reason(Signal::Default, "production host · two gates"),
        );
    }
}

fn workflow_items(w: &World, out: &mut Vec<Item>) {
    let root = w
        .location
        .project
        .as_ref()
        .map(|p| p.root.clone())
        .unwrap_or(w.location.cwd.clone());
    for (name, cmd, trusted) in &w.workflows {
        let id = format!(
            "workflow.{}",
            name.to_lowercase().split_whitespace().next().unwrap_or("x")
        );
        let mut item = Item::new(
            &id,
            name,
            Kind::Personal,
            ResultType::Action,
            tag_for_dir(w, &root),
        )
        .summary(&format!(
            "team workflow from {}/.holla/workflows.toml",
            w.location.short(&root)
        ))
        .commands(&[cmd])
        .keywords(&[
            "workflow",
            "deploy",
            "team",
            "personal",
            &name.to_lowercase(),
        ])
        .group("Tasks")
        .risk(Risk::Mutating)
        .launch(Launch::Activity {
            script: format!("generic:{cmd}"),
        })
        .alt("Set an alias…", &format!("{id}.alias"))
        .reason(Signal::Context, "provenance: project workflow file");
        if !trusted {
            item = item.confirm(Confirmation::Trust {
                config: format!("{}/.holla/workflows.toml", w.location.short(&root)),
            });
        }
        out.push(item);
    }
}

fn file_items(w: &World, out: &mut Vec<Item>) {
    let here = ScopeTag::here(&w.location.cwd);
    if let Some(cfg) = w.mise.configs.first() {
        out.push(
            Item::new(
                "file.mise",
                &format!(
                    "Open {}",
                    cfg.path.rsplit('/').next().unwrap_or("mise.toml")
                ),
                Kind::Files,
                ResultType::Resource,
                tag_for_dir(
                    w,
                    cfg.path
                        .rsplit_once('/')
                        .map(|(d, _)| d)
                        .unwrap_or(&w.location.cwd),
                ),
            )
            .summary(&format!(
                "open {} in $EDITOR · {} tasks · {}",
                w.location.short(&cfg.path),
                cfg.tasks.len(),
                if cfg.trusted {
                    "trusted"
                } else {
                    "not trusted"
                }
            ))
            .commands(&[&format!("$EDITOR {}", cfg.path)])
            .keywords(&["open", "config", "mise.toml", "edit", "file"])
            .group("Files")
            .launch(Launch::Insert)
            .reason(Signal::Default, "config here"),
        );
    }
    if let Some(p) = &w.location.project {
        let manifest = match p.kind {
            crate::domain::context::ProjectKind::Rust { .. } => Some("Cargo.toml"),
            crate::domain::context::ProjectKind::Node { .. } => Some("package.json"),
            crate::domain::context::ProjectKind::Gradle => Some("settings.gradle.kts"),
            crate::domain::context::ProjectKind::Python => Some("pyproject.toml"),
            _ => None,
        };
        if let Some(m) = manifest {
            out.push(
                Item::new(
                    "file.manifest",
                    &format!("Open {m}"),
                    Kind::Files,
                    ResultType::Resource,
                    tag_for_dir(w, &p.root),
                )
                .summary(&format!(
                    "open {}/{m} in $EDITOR",
                    w.location.short(&p.root)
                ))
                .commands(&[&format!("$EDITOR {}/{m}", p.root)])
                .keywords(&["open", "config", m, "manifest", "edit"])
                .group("Files")
                .launch(Launch::Insert)
                .reason(Signal::Default, "manifest here"),
            );
        }
    }
    if w.docker.compose.is_some() || w.host.role == crate::domain::context::HostRole::Production {
        let (path, size) = if w.host.role == crate::domain::context::HostRole::Production {
            ("/var/log/payments/payments.log", "212 MB")
        } else {
            ("logs/api.log", "2.1 MB")
        };
        out.push(
            Item::new(
                "file.log",
                &format!("Open {path}"),
                Kind::Files,
                ResultType::Resource,
                here.clone(),
            )
            .summary(&format!("nearby log file · {size} · opens in the pager"))
            .commands(&[&format!("less +F {path}")])
            .keywords(&["log", "logs", "file", "api.log", "tail"])
            .group("Files")
            .launch(Launch::Activity {
                script: format!("generic:less +F {path}"),
            })
            .reason(Signal::Default, &format!("{size} nearby")),
        );
    }
    out.push(
        Item::new(
            "file.reveal",
            "Copy this folder's path",
            Kind::Files,
            ResultType::Action,
            here,
        )
        .summary(&format!("copies {} to the clipboard", w.location.cwd))
        .commands(&[&format!("printf %s {} | pbcopy", w.location.cwd)])
        .keywords(&["copy", "path", "pwd", "folder", "clipboard"])
        .group("Files")
        .launch(Launch::Insert)
        .reason(Signal::Default, "available"),
    );
}

fn activity_items(w: &World, out: &mut Vec<Item>) {
    for a in &w.activities {
        let state = a.state.label();
        out.push(
            Item::new(
                &format!("activity.{}", a.id),
                &a.name,
                Kind::Activity,
                ResultType::Resource,
                a.scope.clone(),
            )
            .summary(&format!(
                "{} · started by {} · {} lines retained",
                state,
                a.origin,
                a.output.len()
            ))
            .commands(&[])
            .keywords(&["activity", "running", "tab", &a.name, state])
            .group("Activities")
            .launch(Launch::OpenActivity { id: a.id.clone() })
            .alt("Stop", &format!("activity.{}.stop", a.id))
            .alt("Restart", &format!("activity.{}.restart", a.id))
            .reason(
                if a.state.live() {
                    Signal::Context
                } else {
                    Signal::Default
                },
                &format!(
                    "{} {state} · {}",
                    match a.state {
                        crate::domain::activity::ActivityState::Running
                        | crate::domain::activity::ActivityState::Detached => "⠋",
                        crate::domain::activity::ActivityState::Succeeded => "✓",
                        crate::domain::activity::ActivityState::Failed => "!",
                        crate::domain::activity::ActivityState::Stopped => "○",
                    },
                    crate::screens::ticks_label(a.duration_ticks(w.tick))
                ),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::fixtures::world_for;
    use crate::domain::ranking::search;
    use crate::scenario::{Motion, Scenario};

    fn labels(w: &World, q: &str, scope: Scope) -> Vec<String> {
        let items = w.items();
        search(&items, q, scope, None)
            .iter()
            .map(|r| items[r.index].label.clone())
            .collect()
    }

    #[test]
    fn rust_dirty_suggests_review_tests_and_pull_with_reasons() {
        let w = world_for(Scenario::RustDirty, Motion::Reduced);
        let items = w.items();
        let top: Vec<&Item> = search(&items, "", Scope::Here, None)
            .iter()
            .take(6)
            .map(|r| &items[r.index])
            .collect();
        let labels: Vec<&str> = top.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"Review 5 modified files"), "{labels:?}");
        assert!(labels.contains(&"Run tests"));
        assert!(labels.contains(&"Pull"));
        let pull = items.iter().find(|i| i.id == "git.pull").unwrap();
        assert_eq!(
            pull.top_reason().unwrap().text,
            "branch is 3 commits behind"
        );
        let tests = items.iter().find(|i| i.id == "mise.task.test").unwrap();
        assert_eq!(tests.used_here, 6);
        assert!(tests.aliases.contains(&"test".to_owned()));
    }

    #[test]
    fn monorepo_child_ranks_local_first_and_keeps_parent_ecosystem_rows() {
        let w = world_for(Scenario::MonorepoChild, Motion::Reduced);
        let items = w.items();
        let eco = items
            .iter()
            .find(|i| i.label == "Start development ecosystem")
            .expect("parent task");
        assert_eq!(eco.scope.direction, Scope::Parent);
        assert!(eco.scope.runs_in.ends_with("/acme"));
        let dev = items
            .iter()
            .find(|i| i.id == "mise.task.//apps/frontend:dev")
            .unwrap();
        assert_eq!(dev.scope.direction, Scope::Here);
        assert!(
            matches!(dev.confirmation, Confirmation::Trust { .. }),
            "untrusted child config"
        );
        let l = labels(&w, "", Scope::Parent);
        assert!(l.iter().any(|x| x == "Start required containers"), "{l:?}");
    }

    #[test]
    fn docker_cleanup_is_first_for_its_query_and_two_gated() {
        let w = world_for(Scenario::DockerCleanup, Motion::Reduced);
        let l = labels(&w, "docker clean", Scope::Here);
        assert_eq!(l[0], "Clean Docker completely", "{l:?}");
        assert!(l.contains(&"Stop all containers".to_owned()));
        assert!(l.contains(&"Prune builder cache".to_owned()));
        let items = w.items();
        let c = items.iter().find(|i| i.id == "docker.cleanup").unwrap();
        assert_eq!(
            c.confirmation.phrase().as_deref(),
            Some("I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox")
        );
        let d = items.iter().find(|i| i.id == "disk.delete_all").unwrap();
        assert_eq!(
            d.confirmation.phrase().as_deref(),
            Some("DELETE EVERYTHING IN /work/scratch")
        );
    }

    #[test]
    fn remote_host_rows_say_the_host_and_restart_needs_two_gates() {
        let w = world_for(Scenario::RemoteHost, Motion::Reduced);
        let items = w.items();
        let r = items
            .iter()
            .find(|i| i.id == "service.restart.payments")
            .unwrap();
        assert_eq!(r.scope.word, "on prod-eu-1");
        assert_eq!(
            r.confirmation.phrase().as_deref(),
            Some("RESTART PAYMENTS ON prod-eu-1")
        );
        let l = labels(&w, "", Scope::Here);
        assert!(l.iter().any(|x| x.contains("payments-worker")), "{l:?}");
        assert!(
            !items.iter().any(|i| i.id.starts_with("docker.")),
            "no docker on the host means no docker rows"
        );
        let hard = world_for(Scenario::HardCases, Motion::Reduced);
        assert!(
            hard.items().iter().any(
                |i| i.id == "docker.daemon" && matches!(i.freshness, Freshness::Unavailable(_))
            ),
            "an unreachable daemon stays visible as an unavailable resource"
        );
    }

    #[test]
    fn exact_keywords_beat_scattered_subsequences() {
        let w = world_for(Scenario::RustDirty, Motion::Reduced);
        let l = labels(&w, "port", Scope::Here);
        assert_eq!(l[0], "Find the process on a port…", "{l:?}");
        let l = labels(&w, "clone", Scope::Here);
        assert_eq!(l[0], "Clone a GitHub repository…", "{l:?}");
        let l = labels(&w, "gp", Scope::Here);
        assert_eq!(l[0], "Pull");
    }

    #[test]
    fn intent_queries_cross_domains() {
        let w = world_for(Scenario::MonorepoRoot, Motion::Reduced);
        let l = labels(&w, "logs", Scope::Here);
        assert!(
            l.iter().any(|x| x.starts_with("Follow logs from api")),
            "{l:?}"
        );
        assert!(
            l.iter().any(|x| x.starts_with("Follow acme-api-1 logs")),
            "{l:?}"
        );
        assert!(l.iter().any(|x| x.contains("api.log")), "{l:?}");
        let w = world_for(Scenario::DiskCleanup, Motion::Reduced);
        let l = labels(&w, "checkout main", Scope::Here);
        assert!(l.iter().any(|x| x.contains("primary branch")), "{l:?}");
        let l = labels(&w, "sync projects", Scope::Here);
        assert_eq!(l[0], "Pull every child project", "{l:?}");
    }
}
