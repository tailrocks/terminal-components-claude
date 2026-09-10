//! Snapshot pages: read-only views built from the fixture world (git
//! status, system resources, the PostgreSQL blocking tree, Docker
//! accounting, mise configuration, SSH resolution…) with follow-up actions.
//! They are read, not run, so they are pages, never tabs (D-15).

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::text::{truncate, width};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::{Meter, MeterTone, MeterVisual};
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::statusbar::StatusItem;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use crate::screens::{Cx, Go, Screen, StatusBits, plural};
use crate::sim::world::World;

pub const BODY: WidgetId = WidgetId::of("snapshot.body");

/// A follow-up action: label, item id, arguments.
type FollowUp = (String, String, Vec<(String, String)>);
const FOLLOW: WidgetId = WidgetId::of("snapshot.follow");

pub struct SnapshotPage {
    pub kind: String,
    title: String,
    meta: String,
    facts: Vec<Prop>,
    lines: Vec<(String, Tone)>,
    /// (label, item id, args)
    follow_ups: Vec<FollowUp>,
    buttons: Vec<Button>,
    scroll: ScrollState,
    meters: Vec<(String, u8)>,
    detail_title: String,
    /// The report shown here was still being written on the last tick.
    report_running: bool,
}

fn t(s: &str) -> (String, Tone) {
    (s.to_owned(), Tone::Normal)
}
fn m(s: &str) -> (String, Tone) {
    (s.to_owned(), Tone::Muted)
}
fn e(s: &str) -> (String, Tone) {
    (s.to_owned(), Tone::Error)
}
fn wn(s: &str) -> (String, Tone) {
    (s.to_owned(), Tone::Warning)
}

impl SnapshotPage {
    pub fn new(kind: &str, w: &World) -> Self {
        let mut page = Self {
            kind: kind.into(),
            title: String::new(),
            meta: String::new(),
            facts: vec![],
            lines: vec![],
            follow_ups: vec![],
            buttons: vec![],
            scroll: ScrollState::default(),
            meters: vec![],
            detail_title: "Detail".into(),
            report_running: false,
        };
        page.build(w);
        page.detail_title = match kind {
            "git" | "git-diff" | "git-stash" => "Status",
            "git-all" => "Projects",
            "system" | "processes" => "Processes",
            "docker-ps" | "compose-ps" | "docker-df" => "Containers",
            "docker-unavailable" => "Daemon",
            "pg" | "pg-blocking" => "Sessions · blockers first",
            "mise" | "mise-outdated" => "Tools and tasks",
            "disk-history" => "History",
            k if k.starts_with("service-") => "Journal",
            k if k.starts_with("ssh:") => "ssh -G",
            k if k.starts_with("port:") => "Listeners",
            _ => "Detail",
        }
        .into();
        page.buttons = page
            .follow_ups
            .iter()
            .enumerate()
            .map(|(i, (l, _, _))| Button::subtle(FOLLOW.child(i), l))
            .collect();
        page
    }

    /// Re-read the world: a report that is still being written changes
    /// every tick.
    fn rebuild(&mut self, w: &World) {
        self.facts.clear();
        self.lines.clear();
        self.meters.clear();
        self.follow_ups.clear();
        self.build(w);
        self.buttons = self
            .follow_ups
            .iter()
            .enumerate()
            .map(|(i, (l, _, _))| Button::subtle(FOLLOW.child(i), l))
            .collect();
    }

    fn report_index(&self) -> Option<usize> {
        self.kind.strip_prefix("report:")?.parse().ok()
    }

    fn build(&mut self, w: &World) {
        let host = w.host.name.clone();
        let mut fu: Vec<FollowUp> = vec![];
        match self.kind.as_str() {
            "git" | "git-diff" | "git-stash" => {
                if let Some(g) = w.git_here() {
                    self.title = if self.kind == "git-diff" {
                        "Modified files".into()
                    } else {
                        "Git status".into()
                    };
                    self.meta = w.location.short(&g.path);
                    self.facts = vec![
                        Prop::new("Branch", g.branch.clone().unwrap_or_else(|| format!("detached @ {}", g.head_short))),
                        Prop::new("Upstream", match &g.upstream {
                            Some(u) => format!("{u} · {} behind · {} ahead", g.behind, g.ahead),
                            None => "none".into(),
                        }),
                        Prop::new("Primary", format!("{} · resolved from origin/HEAD", g.primary)).tone(Tone::Muted),
                        Prop::new("Worktree", format!("{} staged · {} unstaged · {} untracked · {} conflicts · {} stashed", g.staged, g.unstaged, g.untracked, g.conflicts, g.stash)).tone(if g.dirty() { Tone::Warning } else { Tone::Normal }),
                    ];
                    if let Some(op) = &g.in_progress {
                        self.facts
                            .push(Prop::new("In progress", op.clone()).tone(Tone::Error));
                    }
                    self.lines.push(m(&format!("$ {}", g.status_cmd())));
                    for f in &g.modified {
                        self.lines.push(t(&format!(" M {f}")));
                    }
                    if g.untracked > 0 {
                        self.lines
                            .push(t(&format!("?? … {} untracked", g.untracked)));
                    }
                    if g.conflicts > 0 {
                        self.lines
                            .push(e(&format!("UU … {} conflicting", g.conflicts)));
                    }
                    if self.kind == "git-diff" {
                        self.lines.push(m(""));
                        self.lines.push(m("$ git diff --stat"));
                        for (i, f) in g.modified.iter().enumerate() {
                            self.lines.push(t(&format!(
                                " {f:<48} | {:>3} {}",
                                12 + i * 9,
                                "+".repeat(3 + i)
                            )));
                        }
                        self.lines
                            .push(t(&format!(" {} files changed", g.modified.len())));
                    }
                    if g.behind > 0 {
                        fu.push(("Pull".into(), "git.pull".into(), vec![]));
                    }
                    fu.push(("Open lazygit".into(), "git.tui".into(), vec![]));
                }
            }
            "git-all" => {
                self.title = "Status across child projects".into();
                self.meta = w.location.cwd_short();
                let n = w.git.iter().filter(|g| g.worktree_of.is_none()).count();
                self.facts = vec![
                    Prop::new(
                        "Projects",
                        format!(
                            "{n} · {} worktree deduplicated · {} submodule",
                            w.git.iter().filter(|g| g.worktree_of.is_some()).count(),
                            w.git.iter().filter(|g| g.submodule).count()
                        ),
                    ),
                    Prop::new("Inspected", "in parallel · 4 at a time").tone(Tone::Muted),
                ];
                self.lines.push(m(&format!(
                    "{:<14}{:<12}{:<20}{}",
                    "project", "branch", "state", "eligibility"
                )));
                for g in &w.git {
                    let name = g.path.rsplit('/').next().unwrap_or("").to_owned();
                    let kind = if g.submodule {
                        " (submodule)"
                    } else if g.worktree_of.is_some() {
                        " (worktree)"
                    } else {
                        ""
                    };
                    let elig = match g.block_reason() {
                        Some(r) => format!("blocked · {r}"),
                        None => "pull and switch eligible".into(),
                    };
                    let tone = if g.block_reason().is_some() {
                        Tone::Warning
                    } else {
                        Tone::Normal
                    };
                    self.lines.push((
                        format!(
                            "{:<14}{:<12}{:<20}{}",
                            format!("{name}{kind}"),
                            g.branch.clone().unwrap_or("detached".into()),
                            g.summary(),
                            elig
                        ),
                        tone,
                    ));
                }
                fu.push((
                    "Pull every child project".into(),
                    "git.pull_all".into(),
                    vec![],
                ));
                fu.push((
                    "Switch every project to its primary branch".into(),
                    "git.switch_all".into(),
                    vec![],
                ));
            }
            "system" | "processes" => {
                let s = &w.system;
                self.title = if self.kind == "system" {
                    "System resources".into()
                } else {
                    "Process tree".into()
                };
                self.meta = format!("{host} · {}", w.host.os.label());
                self.meters = vec![
                    ("CPU".into(), s.cpu_pct as u8),
                    (
                        "Memory".into(),
                        (s.mem_used_gb / s.mem_total_gb * 100.0) as u8,
                    ),
                ];
                if let Some(fs) = w.disk.root_fs() {
                    self.meters.push((format!("Disk {}", fs.mount), fs.pct()));
                }
                self.facts = vec![
                    Prop::new("Load", s.load.clone()),
                    Prop::new(
                        "Cores",
                        s.cores
                            .iter()
                            .map(|c| format!("{c}%"))
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                    .tone(Tone::Muted),
                    Prop::new(
                        "Memory",
                        format!(
                            "{:.1} of {:.0} GB · swap {:.1} GB",
                            s.mem_used_gb, s.mem_total_gb, s.swap_used_gb
                        ),
                    ),
                    Prop::new(
                        "Pressure",
                        match s.pressure {
                            Some((some, full)) => format!(
                                "cpu some {some}% · full {full}% · {} min sustained",
                                s.pressure_minutes
                            ),
                            None if s.pressure_minutes > 0 => format!(
                                "{} min of sustained CPU pressure (no PSI on {})",
                                s.pressure_minutes,
                                w.host.os.label()
                            ),
                            None => "none".into(),
                        },
                    )
                    .tone(if s.pressure_minutes > 0 {
                        Tone::Warning
                    } else {
                        Tone::Muted
                    }),
                ];
                for (d, v) in &s.disk_io {
                    self.facts
                        .push(Prop::new("Disk I/O", format!("{d} · {v}")).tone(Tone::Muted));
                }
                for (n, v) in &s.net {
                    self.facts
                        .push(Prop::new("Network", format!("{n} · {v}")).tone(Tone::Muted));
                }
                if let Some(c) = s.temp_c {
                    self.facts
                        .push(Prop::new("Temperature", format!("{c} °C")).tone(if c > 80 {
                            Tone::Warning
                        } else {
                            Tone::Muted
                        }));
                }
                self.lines.push(m(&format!(
                    "{:<8}{:<20}{:>6}{:>9}   {}",
                    "pid", "name", "cpu", "mem", "state"
                )));
                for p in &s.top {
                    let tone = if p.cpu_pct >= 30 {
                        Tone::Warning
                    } else {
                        Tone::Normal
                    };
                    self.lines.push((
                        format!(
                            "{:<8}{:<20}{:>5}%{:>7} MB   {}{}",
                            p.pid,
                            p.name,
                            p.cpu_pct,
                            p.mem_mb,
                            p.state,
                            p.port.map(|x| format!(" · port {x}")).unwrap_or_default()
                        ),
                        tone,
                    ));
                }
                if s.btm_installed {
                    fu.push(("Open btm".into(), "system.monitor".into(), vec![]));
                }
                fu.push(("Stop a process…".into(), "system.kill".into(), vec![]));
            }
            k if k.starts_with("port:") => {
                let port: u16 = k.trim_start_matches("port:").parse().unwrap_or(5173);
                self.title = format!("Port {port}");
                self.meta = host.clone();
                self.lines
                    .push(m(&format!("$ lsof -nP -iTCP:{port} -sTCP:LISTEN")));
                let procs: Vec<&crate::domain::stack::Proc> = w
                    .system
                    .top
                    .iter()
                    .filter(|p| p.port == Some(port))
                    .collect();
                if procs.is_empty() {
                    self.facts.push(
                        Prop::new("Result", "nothing listens on this port").tone(Tone::Muted),
                    );
                } else {
                    self.lines.push(m(&format!(
                        "{:<20}{:<8}{:<10}{}",
                        "COMMAND", "PID", "USER", "NAME"
                    )));
                    for p in procs {
                        self.lines.push(t(&format!(
                            "{:<20}{:<8}{:<10}*:{port} (LISTEN)",
                            p.name, p.pid, w.host.user
                        )));
                        self.facts.push(Prop::new(
                            "Process",
                            format!("{} · pid {} · {} MB · {}", p.name, p.pid, p.mem_mb, p.state),
                        ));
                        fu.push((
                            format!("Stop {} (pid {})", p.name, p.pid),
                            "system.kill".into(),
                            vec![
                                ("pid".into(), p.pid.to_string()),
                                ("signal".into(), "TERM".into()),
                            ],
                        ));
                    }
                }
            }
            "docker-ps" | "docker-df" | "compose-ps" | "docker-unavailable" => {
                let d = &w.docker;
                match &d.daemon {
                    Err(r) => {
                        self.title = "Docker".into();
                        self.meta = host.clone();
                        self.facts = vec![Prop::new("Daemon", r.clone()).tone(Tone::Error), Prop::new("Capability", "docker CLI present · daemon unreachable · container actions hidden until it answers").tone(Tone::Muted)];
                        self.lines.push(e("$ docker info"));
                        self.lines.push(e(r));
                    }
                    Ok(()) => {
                        self.title = match self.kind.as_str() {
                            "docker-df" => "Docker disk usage".into(),
                            "compose-ps" => format!(
                                "Compose · {}",
                                d.compose
                                    .as_ref()
                                    .map(|c| c.name.clone())
                                    .unwrap_or_default()
                            ),
                            _ => "Containers".into(),
                        };
                        self.meta = host.clone();
                        self.facts = vec![
                            Prop::new(
                                "Containers",
                                format!("{} · {} running", d.containers.len(), d.running()),
                            ),
                            Prop::new(
                                "Images",
                                format!(
                                    "{} · {:.1} GB · {} dangling",
                                    d.images, d.images_gb, d.dangling_images
                                ),
                            ),
                            Prop::new(
                                "Volumes",
                                format!(
                                    "{} · {} named · {:.1} GB",
                                    d.volumes.len(),
                                    d.named_volumes(),
                                    d.volumes_gb()
                                ),
                            ),
                            Prop::new("Build cache", format!("{:.1} GB", d.builder_cache_gb)),
                            Prop::new("Reclaimable", format!("~{:.1} GB", d.reclaimable_gb))
                                .tone(Tone::Warning),
                        ];
                        self.lines.push(m(&format!(
                            "{:<20}{:<24}{:<10}{:<12}{}",
                            "name", "image", "state", "ports", "project"
                        )));
                        for c in &d.containers {
                            let state = if c.running {
                                c.health.clone().unwrap_or("running".into())
                            } else {
                                "exited".into()
                            };
                            let tone = match state.as_str() {
                                "unhealthy" => Tone::Error,
                                "exited" => Tone::Muted,
                                _ => Tone::Normal,
                            };
                            self.lines.push((
                                format!(
                                    "{:<20}{:<24}{:<10}{:<12}{}",
                                    c.name,
                                    truncate(&c.image, 22),
                                    state,
                                    c.ports,
                                    c.project.clone().unwrap_or("–".into())
                                ),
                                tone,
                            ));
                        }
                        fu.push((
                            "Clean Docker completely".into(),
                            "docker.cleanup".into(),
                            vec![],
                        ));
                        fu.push((
                            "Stop all containers".into(),
                            "docker.stop_all".into(),
                            vec![],
                        ));
                    }
                }
            }
            "pg" | "pg-blocking" => {
                if let Some(p) = &w.pg {
                    self.title = if self.kind == "pg" {
                        "PostgreSQL activity".into()
                    } else {
                        "Who is blocking the database?".into()
                    };
                    self.meta = p.label.clone();
                    self.meters = vec![(
                        "Connections".into(),
                        (p.connections * 100 / p.max_connections.max(1)) as u8,
                    )];
                    self.facts = vec![
                        Prop::new("Source", p.source.clone()).tone(Tone::Muted),
                        Prop::new(
                            "Connections",
                            format!("{} of {}", p.connections, p.max_connections),
                        ),
                        Prop::new("Idle in txn", format!("{}", p.idle_in_txn())).tone(
                            if p.idle_in_txn() > 0 {
                                Tone::Warning
                            } else {
                                Tone::Normal
                            },
                        ),
                        Prop::new("Blocked", format!("{}", p.blocked().len())).tone(
                            if p.blocked().is_empty() {
                                Tone::Normal
                            } else {
                                Tone::Error
                            },
                        ),
                        Prop::new(
                            "Cache hit",
                            format!(
                                "{}% · {} deadlocks in 24 h",
                                p.cache_hit_pct, p.deadlocks_24h
                            ),
                        )
                        .tone(Tone::Muted),
                    ];
                    if let Some(pid) = w.pg_blocker() {
                        self.facts.push(
                            Prop::new(
                                "Blocker",
                                format!("pid {pid} · cancel or terminate it to free the waiters"),
                            )
                            .tone(Tone::Error)
                            .wrap(),
                        );
                    }
                    if let Some(l) = &p.replication_lag {
                        self.facts
                            .push(Prop::new("Replication", l.clone()).tone(Tone::Warning));
                    }
                    // blocking tree: blockers first, their victims indented
                    self.lines.push(m(&format!(
                        "{:<8}{:<20}{:<10}{:<14}{:>8}   {}",
                        "pid", "state", "user", "wait", "txn", "query"
                    )));
                    for b in p.blockers() {
                        let row = |indent: &str, s: &crate::domain::stack::PgSession| {
                            format!(
                                "{:<8}{:<20}{:<10}{:<14}{:>7}s   {}",
                                format!("{indent}{}", s.pid),
                                s.state,
                                s.user,
                                s.wait.clone().unwrap_or("–".into()),
                                s.txn_secs.max(s.query_secs),
                                truncate(&s.query, 40)
                            )
                        };
                        self.lines.push(wn(&row("", b)));
                        for v in p.sessions.iter().filter(|s| s.blocked_by == Some(b.pid)) {
                            self.lines.push(e(&row("└ ", v)));
                            for vv in p.sessions.iter().filter(|s| s.blocked_by == Some(v.pid)) {
                                self.lines.push(e(&row("  └ ", vv)));
                            }
                        }
                    }
                    for s in p.sessions.iter().filter(|s| {
                        s.blocked_by.is_none() && !p.blockers().iter().any(|b| b.pid == s.pid)
                    }) {
                        self.lines.push(t(&format!(
                            "{:<8}{:<20}{:<10}{:<14}{:>7}s   {}",
                            s.pid,
                            s.state,
                            s.user,
                            s.wait.clone().unwrap_or("–".into()),
                            s.txn_secs,
                            truncate(&s.query, 40)
                        )));
                    }
                    if let Some(b) = p.blockers().first() {
                        fu.push((
                            format!("Cancel the blocking query (PID {})", b.pid),
                            "pg.cancel".into(),
                            vec![],
                        ));
                        fu.push((
                            format!("Terminate backend {}", b.pid),
                            "pg.terminate".into(),
                            vec![],
                        ));
                    }
                    if p.pg_activity_installed {
                        fu.push(("Open pg_activity".into(), "pg.handoff".into(), vec![]));
                    }
                }
            }
            "mise" | "mise-outdated" => {
                let mi = &w.mise;
                self.title = if self.kind == "mise" {
                    "mise configuration".into()
                } else {
                    "Outdated mise tools".into()
                };
                self.meta = format!("mise {}", mi.version);
                self.facts = vec![
                    Prop::new(
                        "Chain",
                        mi.configs
                            .iter()
                            .map(|c| w.location.short(&c.path))
                            .collect::<Vec<_>>()
                            .join(" › "),
                    )
                    .wrap(),
                    Prop::new(
                        "Monorepo",
                        mi.monorepo_root
                            .as_ref()
                            .map(|r| w.location.short(r))
                            .unwrap_or("none".into()),
                    )
                    .tone(Tone::Muted),
                    Prop::new(
                        "Trust",
                        mi.configs
                            .iter()
                            .map(|c| {
                                format!(
                                    "{} {}",
                                    c.path.rsplit('/').nth(1).unwrap_or(""),
                                    if c.trusted || w.trusted_now.contains(&c.path) {
                                        "trusted"
                                    } else {
                                        "not trusted"
                                    }
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" · "),
                    )
                    .wrap(),
                ];
                self.lines.push(m(&format!(
                    "{:<14}{:<10}{:<12}{:<12}{}",
                    "tool", "requested", "active", "latest", "scope"
                )));
                for c in &mi.configs {
                    for tl in &c.tools {
                        let tone = if !tl.installed {
                            Tone::Error
                        } else if tl.active.as_deref() != Some(tl.latest.as_str()) {
                            Tone::Warning
                        } else {
                            Tone::Normal
                        };
                        self.lines.push((
                            format!(
                                "{:<14}{:<10}{:<12}{:<12}{}",
                                tl.name,
                                tl.requested,
                                tl.active.clone().unwrap_or("missing".into()),
                                tl.latest,
                                w.location.short(&c.path)
                            ),
                            tone,
                        ));
                    }
                }
                for tl in &mi.global_tools {
                    let tone = if tl.active.as_deref() != Some(tl.latest.as_str()) {
                        Tone::Warning
                    } else {
                        Tone::Normal
                    };
                    self.lines.push((
                        format!(
                            "{:<14}{:<10}{:<12}{:<12}global",
                            tl.name,
                            tl.requested,
                            tl.active.clone().unwrap_or("missing".into()),
                            tl.latest
                        ),
                        tone,
                    ));
                }
                if self.kind == "mise" {
                    self.lines.push(m(""));
                    self.lines
                        .push(m(&format!("{:<40}{:<24}{}", "task", "runs in", "depends")));
                    for c in &mi.configs {
                        for tk in &c.tasks {
                            self.lines.push(t(&format!(
                                "{:<40}{:<24}{}",
                                truncate(&tk.namespaced, 38),
                                truncate(&w.location.short(&tk.dir), 22),
                                tk.depends.join(", ")
                            )));
                        }
                    }
                }
                if !mi.outdated().is_empty() {
                    fu.push((
                        "Upgrade outdated tools".into(),
                        "upgrade.mise".into(),
                        vec![],
                    ));
                }
            }
            "disk-history" => {
                self.title = "Cleanup history".into();
                self.meta = host.clone();
                self.facts = vec![
                    Prop::new(
                        "Entries",
                        format!(
                            "{} · {} operation log records",
                            w.disk.history.len()
                                + w
                                    .ops_log
                                    .lines
                                    .iter()
                                    .filter(|l| crate::domain::cleanup::LogRecord::parse(l).is_some())
                                    .count(),
                            w.ops_log.lines.len()
                        ),
                    ),
                    Prop::new("Log", format!("{} · JSONL v1 · append only", w.location.short(&w.ops_log.path))).tone(Tone::Muted).wrap(),
                    Prop::new("Audit", "targets, method, reclaimed space, skips and failures · restoration only from Trash").tone(Tone::Muted),
                ];
                if let Some(e) = &w.ops_log.write_failure {
                    self.facts.push(
                        Prop::new(
                            "Log failure",
                            format!("{e} · deletions still ran · nothing rolled back"),
                        )
                        .tone(Tone::Error)
                        .wrap(),
                    );
                }
                for (i, r) in w.reports.iter().enumerate() {
                    fu.push((
                        format!("Report {} · {}", i + 1, truncate(&r.summary(), 40)),
                        format!("report:{i}"),
                        vec![],
                    ));
                }
                if !w.ops_log.lines.is_empty() {
                    self.lines.push(m("operation log · newest last"));
                    for l in w
                        .ops_log
                        .lines
                        .iter()
                        .rev()
                        .take(50)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                    {
                        let tone = if l.contains("\"outcome\":\"failed\"") {
                            Tone::Error
                        } else if l.contains("\"outcome\":\"skipped\"") {
                            Tone::Warning
                        } else {
                            Tone::Normal
                        };
                        self.lines.push((l.clone(), tone));
                    }
                    self.lines.push(m(""));
                }
                // one table over both sources: command cleanups the world
                // recorded and every parsed operation log record
                let mut rows: Vec<(i64, String, String, String, String, Tone)> = w
                    .disk
                    .history
                    .iter()
                    .map(|h| {
                        (
                            h.when_secs,
                            h.target.clone(),
                            h.method.clone(),
                            format!("{:.1} GB", h.reclaimed_gb),
                            h.outcome.clone(),
                            Tone::Normal,
                        )
                    })
                    .collect();
                for l in &w.ops_log.lines {
                    let Some(r) = crate::domain::cleanup::LogRecord::parse(l) else {
                        continue;
                    };
                    let method = if r.dry_run {
                        "dry run".to_owned()
                    } else {
                        r.mode.label().to_owned()
                    };
                    let (outcome, tone) = match r.outcome {
                        crate::domain::cleanup::Outcome::Failed => (
                            format!(
                                "failed · {}",
                                r.error.as_deref().unwrap_or("no reason recorded")
                            ),
                            Tone::Error,
                        ),
                        crate::domain::cleanup::Outcome::Skipped => (
                            format!(
                                "skipped · {}",
                                r.error.as_deref().unwrap_or("no reason recorded")
                            ),
                            Tone::Warning,
                        ),
                        crate::domain::cleanup::Outcome::WouldRemove => {
                            ("would be removed".to_owned(), Tone::Muted)
                        }
                        other => (other.label().to_owned(), Tone::Normal),
                    };
                    let freed = if matches!(
                        r.outcome,
                        crate::domain::cleanup::Outcome::Removed
                            | crate::domain::cleanup::Outcome::Trashed
                    ) {
                        crate::sim::fs::human(r.size)
                    } else {
                        "0 B".to_owned()
                    };
                    rows.push((
                        r.timestamp_ms.div_euclid(1000),
                        r.path,
                        method,
                        freed,
                        outcome,
                        tone,
                    ));
                }
                rows.sort_by_key(|r| r.0);
                self.lines.push(m(&format!(
                    "{:<18}{:<44}{:<12}{:>10}   {}",
                    "when", "target", "method", "freed", "outcome"
                )));
                for (when, target, method, freed, outcome, tone) in rows {
                    self.lines.push((
                        format!(
                            "{:<18}{:<44}{:<12}{:>10}   {}",
                            crate::clock::Clock::stamp(when),
                            truncate(&w.location.short(&target), 42),
                            method,
                            freed,
                            outcome
                        ),
                        tone,
                    ));
                }
            }
            k if k.starts_with("service-") => {
                let name = k.trim_start_matches("service-");
                self.title = format!("{name}.service");
                self.meta = format!("◆ {host} · production");
                let healed = w.healed_units.iter().any(|u| u == name);
                let degraded = name == "payments-worker" && !healed;
                self.facts = vec![
                    Prop::new(
                        "Active",
                        if degraded {
                            "activating (auto-restart) · 3 restarts in 10 min"
                        } else if healed {
                            "active (running) · restarted by holla just now"
                        } else {
                            "active (running)"
                        },
                    )
                    .tone(if degraded { Tone::Error } else { Tone::Normal }),
                    Prop::new(
                        "Main PID",
                        if degraded {
                            "41903 · payments-worker"
                        } else {
                            "1288 · payments"
                        },
                    ),
                    Prop::new(
                        "Memory",
                        if degraded {
                            "5.1 GB (limit 6 GB)"
                        } else {
                            "3.9 GB"
                        },
                    )
                    .tone(if degraded {
                        Tone::Warning
                    } else {
                        Tone::Normal
                    }),
                    Prop::new("Since", "Thu 2026-09-10 08:33:45 +07").tone(Tone::Muted),
                ];
                self.lines.push(m(&format!("$ systemctl status {name}")));
                if degraded {
                    self.lines.push(e("Sep 10 08:33:40 payments-worker[41872]: ERROR settlement batch 8812 failed: deadline exceeded"));
                    self.lines.push(t("Sep 10 08:33:40 systemd[1]: payments-worker.service: Main process exited, code=exited, status=1/FAILURE"));
                    self.lines.push(wn("Sep 10 08:33:45 systemd[1]: payments-worker.service: Scheduled restart job, restart counter is at 3."));
                } else {
                    self.lines.push(t(&format!(
                        "Sep 10 06:41:02 systemd[1]: Started {name}.service."
                    )));
                }
                fu.push((
                    format!("Follow {name} journal"),
                    format!("service.journal.{name}"),
                    vec![],
                ));
                fu.push((
                    format!("Restart {name}"),
                    format!("service.restart.{name}"),
                    vec![],
                ));
            }
            k if k.starts_with("ssh:") => {
                let alias = k.trim_start_matches("ssh:");
                if let Some(a) = w.ssh.aliases.iter().find(|a| a.alias == alias) {
                    self.title = format!("ssh {alias} · resolved");
                    self.meta = w.location.short(&a.from_file);
                    self.facts = vec![
                        Prop::new("Destination", format!("{}@{}:{}", a.user, a.host, a.port)),
                        Prop::new("Jump", a.jump.clone().unwrap_or("none".into())),
                        Prop::new(
                            "Identities",
                            format!("{} · identities only", a.identities.join(", ")),
                        )
                        .tone(Tone::Muted),
                        Prop::new(
                            "Forwarding",
                            if a.forwards.is_empty() {
                                "none".into()
                            } else {
                                a.forwards.join(" · ")
                            },
                        ),
                        Prop::new("Host key", a.hostkey.clone()),
                        Prop::new(
                            "Multiplexing",
                            if a.multiplexed {
                                "master connection open · reused"
                            } else {
                                "none"
                            },
                        )
                        .tone(Tone::Muted),
                        Prop::new("Role", a.role.clone()).tone(if a.role == "production" {
                            Tone::Warning
                        } else {
                            Tone::Normal
                        }),
                        Prop::new(
                            "Runs",
                            format!("ssh {alias} · the alias itself, never a rebuilt command"),
                        )
                        .tone(Tone::Muted),
                    ];
                    self.lines.push(m(&format!("$ ssh -G {alias}")));
                    self.lines.push(t(&format!("hostname {}", a.host)));
                    self.lines.push(t(&format!("user {}", a.user)));
                    self.lines.push(t(&format!("port {}", a.port)));
                    if let Some(j) = &a.jump {
                        self.lines.push(t(&format!("proxyjump {j}")));
                    }
                    for i in &a.identities {
                        self.lines.push(t(&format!("identityfile {i}")));
                    }
                    self.lines.push(t("stricthostkeychecking ask"));
                    fu.push((
                        format!("Connect to {alias}"),
                        format!("ssh.{alias}"),
                        vec![],
                    ));
                }
            }
            "brew-services" => {
                self.title = "Homebrew services".into();
                self.meta = host.clone();
                match &w.brew {
                    None => {
                        self.facts = vec![
                            Prop::new("Homebrew", "not on PATH · nothing to list")
                                .tone(Tone::Error),
                        ];
                    }
                    Some(b) => {
                        let parsed: Result<Vec<String>, String> = b
                            .list_json
                            .clone()
                            .and_then(|j| crate::domain::catalog::parse_brew_services(&j));
                        let listed: Vec<String> = parsed.clone().unwrap_or_default();
                        self.facts = vec![
                            Prop::new("Source", match &parsed {
                                Ok(_) => "brew services list --json · live".to_owned(),
                                Err(e) => format!("brew services list failed · {e} · cached names are hints only"),
                            }).tone(if parsed.is_err() { Tone::Error } else { Tone::Normal }).wrap(),
                            Prop::new("Cache", match &b.cache {
                                Some(c) => format!("brew-services-v{} · {} · {} names · hint for the next start", c.version, w.clock.ago(c.fetched_at), c.services.len()),
                                None => "none · the first listing writes one".to_owned(),
                            }).tone(Tone::Muted).wrap(),
                            Prop::new("Services", format!("{}", listed.len())),
                        ];
                        if b.cache_write_fails {
                            self.facts.push(
                                Prop::new(
                                    "Cache write",
                                    "fails on this host · listings still work · nothing is retried",
                                )
                                .tone(Tone::Warning)
                                .wrap(),
                            );
                        }
                        self.lines.push(m(&format!(
                            "{:<28}{:<12}{}",
                            "service", "status", "actions"
                        )));
                        for name in &listed {
                            let status = b
                                .services
                                .iter()
                                .find(|s| &s.name == name)
                                .map(|s| s.status.clone())
                                .unwrap_or("none".into());
                            let tone = match status.as_str() {
                                "started" => Tone::Normal,
                                "error" => Tone::Error,
                                _ => Tone::Muted,
                            };
                            self.lines.push((
                                format!("{:<28}{:<12}start · stop · restart", name, status),
                                tone,
                            ));
                        }
                        for name in listed.iter().take(6) {
                            fu.push((
                                format!("Restart {name}"),
                                format!("brew.service.{name}.restart"),
                                vec![],
                            ));
                        }
                    }
                }
            }
            k if k.starts_with("report:") => {
                let idx: usize = k
                    .trim_start_matches("report:")
                    .parse()
                    .unwrap_or(usize::MAX);
                match w.reports.get(idx) {
                    None => {
                        self.title = "Cleanup report".into();
                        self.facts = vec![
                            Prop::new("Report", "gone · nothing was recorded for this index")
                                .tone(Tone::Error),
                        ];
                    }
                    Some(r) => {
                        let running = w
                            .cleanup_job
                            .as_ref()
                            .filter(|j| j.report_index == idx)
                            .map(|j| (j.exec.completed(), j.exec.total()));
                        // a page built while the job runs must rebuild once
                        // more after it settles
                        self.report_running = running.is_some();
                        self.title = match (running, r.dry_run) {
                            (Some(_), true) => "Dry run running".into(),
                            (Some(_), false) => "Cleanup running".into(),
                            (None, true) => "Dry run report".into(),
                            (None, false) => "Cleanup report".into(),
                        };
                        self.meta = host.clone();
                        let (details, over) = r.details();
                        self.facts = vec![
                            match running {
                                Some((k, t)) => Prop::new(
                                    "Outcome",
                                    format!(
                                        "running · {k} of {t} items · {} so far · quitting waits for the rest",
                                        r.summary()
                                    ),
                                )
                                .tone(Tone::Warning)
                                .wrap(),
                                None => Prop::new("Outcome", r.summary())
                                    .tone(if r.incomplete() {
                                        Tone::Warning
                                    } else {
                                        Tone::Normal
                                    })
                                    .wrap(),
                            },
                            Prop::new(
                                "Mode",
                                format!(
                                    "{}{}",
                                    r.mode.label(),
                                    if r.dry_run {
                                        " · dry run · nothing was touched"
                                    } else {
                                        ""
                                    }
                                ),
                            ),
                            Prop::new(
                                "Bytes",
                                format!(
                                    "{} {} · {} freed now{}",
                                    crate::sim::fs::human(r.bytes),
                                    if r.dry_run {
                                        "would leave the tree"
                                    } else {
                                        "left the tree"
                                    },
                                    crate::sim::fs::human(r.freed_now),
                                    if r.mode == crate::domain::cleanup::Mode::Trash && !r.dry_run {
                                        " · the rest returns when the Trash is emptied"
                                    } else {
                                        ""
                                    }
                                ),
                            )
                            .wrap(),
                            Prop::new(
                                "Log",
                                format!(
                                    "{} · {}",
                                    w.location.short(&r.log_path),
                                    if r.log_failures > 0 {
                                        format!("{} write failures", r.log_failures)
                                    } else {
                                        "every record written".to_owned()
                                    }
                                ),
                            )
                            .tone(if r.log_failures > 0 {
                                Tone::Error
                            } else {
                                Tone::Muted
                            })
                            .wrap(),
                            Prop::new("Revision", format!("{:016x}", r.plan_revision))
                                .tone(Tone::Muted),
                        ];
                        self.lines.push(m(&format!(
                            "{:<14}{:>10}   {}",
                            "outcome", "bytes", "path · reason"
                        )));
                        for it in &r.items {
                            let tone = match it.outcome {
                                crate::domain::cleanup::Outcome::Failed => Tone::Error,
                                crate::domain::cleanup::Outcome::Skipped => Tone::Warning,
                                _ => Tone::Normal,
                            };
                            self.lines.push((
                                format!(
                                    "{:<14}{:>10}   {}{}",
                                    it.outcome.label(),
                                    crate::sim::fs::human(it.bytes),
                                    w.location.short(&it.path),
                                    it.error
                                        .as_ref()
                                        .map(|e| format!(" · {e}"))
                                        .unwrap_or_default()
                                ),
                                tone,
                            ));
                        }
                        if over > 0 {
                            self.lines.push(m(&format!(
                                "{} more with the same outcome · {} shown in detail",
                                over,
                                details.len()
                            )));
                        }
                        fu.push(("Cleanup history".into(), "disk.history".into(), vec![]));
                    }
                }
            }
            k if k.starts_with("config:") => {
                let path = k.trim_start_matches("config:").to_owned();
                self.title = "Custom actions".into();
                self.meta = w.location.short(&path);
                let cfg = w
                    .custom_project
                    .iter()
                    .chain(w.custom_global.iter())
                    .find(|c| c.path == path);
                match cfg {
                    None => {
                        self.facts = vec![
                            Prop::new("File", format!("{} · not present", w.location.short(&path))).tone(Tone::Muted).wrap(),
                            Prop::new("Format", "TOML · [[action]] entries with id, label, argv, danger, confirm, description, keywords, group").wrap(),
                            Prop::new("Trust", "a project file runs nothing until its content digest is approved · the global file is trusted by ownership").tone(Tone::Muted).wrap(),
                        ];
                        self.lines.push(m("# example"));
                        self.lines.push(t("[[action]]"));
                        self.lines.push(t("id = \"deploy\""));
                        self.lines.push(t("label = \"Deploy to staging\""));
                        self.lines
                            .push(t("argv = [\"./scripts/deploy.sh\", \"staging\"]"));
                        self.lines.push(t("danger = \"destructive\""));
                    }
                    Some(c) => {
                        let status = w.trust.status(&c.digest, &c.path, &w.location.cwd);
                        self.facts = vec![
                            Prop::new("Origin", c.origin.label()),
                            Prop::new("Digest", format!("sha256:{}", c.digest)).wrap(),
                            Prop::new("Trust", format!("{status:?}")).wrap(),
                            Prop::new(
                                "Actions",
                                format!(
                                    "{} · {} diagnostics",
                                    c.actions.len(),
                                    c.diagnostics.len()
                                ),
                            ),
                        ];
                        for d in &c.diagnostics {
                            self.lines.push(wn(&d.text()));
                        }
                        for a in &c.actions {
                            self.lines.push(t(&format!(
                                "{:<24}{:<13}{}",
                                a.id,
                                a.danger.label(),
                                crate::domain::exec::Command::from_vec(
                                    a.argv.clone(),
                                    &w.location.cwd,
                                    &host
                                )
                                .display()
                            )));
                        }
                        self.lines.push(m(""));
                        self.lines.extend(c.text.lines().map(m));
                    }
                }
            }
            _ => {
                self.title = self.kind.clone();
            }
        }
        self.follow_ups = fu;
    }
}

impl Screen for SnapshotPage {
    fn on_tick(&mut self, w: &mut World, _cx: &mut Cx) -> Outcome {
        let running = self.report_index().is_some_and(|idx| {
            w.cleanup_job
                .as_ref()
                .is_some_and(|j| j.report_index == idx)
        });
        if running || self.report_running {
            self.report_running = running;
            self.rebuild(w);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if let Some(path) = self.kind.strip_prefix("config:").map(str::to_owned) {
            match key.code {
                KeyCode::Char('r') if key.plain() => {
                    cx.go(Go::Reload);
                    cx.status("Re-reading the configuration files · trust is re-checked by digest");
                    return Outcome::Changed;
                }
                KeyCode::Char('u') if key.plain() => {
                    let digest = w
                        .custom_project
                        .iter()
                        .chain(w.custom_global.iter())
                        .find(|c| c.path == path)
                        .map(|c| c.digest.clone());
                    match digest {
                        Some(d) if w.trust.revoke(&d) => {
                            w.persisted.trust = Some(w.trust.serialize());
                            cx.status("Trust revoked · the file must be reviewed again before anything runs");
                            self.build(w);
                        }
                        _ => cx.status("Nothing to revoke · the file is not trusted"),
                    }
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        for i in 0..self.buttons.len() {
            if cx.focus.is(self.buttons[i].id) {
                let (o, fired) = self.buttons[i].on_key(key);
                if fired {
                    let (_, id, args) = self.follow_ups[i].clone();
                    cx.go(Go::Run { item: id, args });
                    return Outcome::Changed;
                }
                if o.consumed() {
                    return o;
                }
            }
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll.scroll_by(-1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.scroll.page_up();
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.scroll.page_down();
                Outcome::Changed
            }
            KeyCode::Char('y') => {
                cx.copy(
                    self.lines
                        .iter()
                        .map(|(l, _)| l.clone())
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
                Outcome::Changed
            }
            KeyCode::Char('r') => {
                cx.status("Refreshed · live");
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, _pos: Position, _w: &mut World, cx: &mut Cx) -> Outcome {
        for i in 0..self.buttons.len() {
            if self.buttons[i].id == id {
                cx.focus.focus(id);
                if self.buttons[i].on_click() {
                    let (_, item, args) = self.follow_ups[i].clone();
                    cx.go(Go::Run { item, args });
                }
                return Outcome::Changed;
            }
        }
        if id == BODY {
            cx.focus.focus(BODY);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == BODY {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        buf.set_string(area.x + 1, area.y, &self.title, t.title());
        let meta = format!("{} · live · {}", self.meta, w.clock.ago(w.now_secs() - 1));
        let mw = width(&meta) as u16;
        if mw + width(&self.title) as u16 + 4 < area.width {
            buf.set_string(
                area.right().saturating_sub(mw + 1),
                area.y,
                &meta,
                t.muted(),
            );
        }
        let mut y = area.y + 2;
        // meters (system, postgres)
        for (label, pct) in &self.meters {
            buf.set_string(area.x + 1, y, label, t.muted());
            let mx = area.x + 14;
            Meter::new(Some(*pct))
                .value(format!("{pct}%"))
                .tone(MeterTone::Normal)
                .visual(MeterVisual::Block)
                .render(
                    Rect::new(mx, y, 34.min(area.width.saturating_sub(16)), 1),
                    buf,
                    ctx,
                    t.canvas,
                );
            y += 1;
        }
        if !self.meters.is_empty() {
            y += 1;
        }
        let facts_area = Rect::new(
            area.x + 1,
            y,
            area.width.saturating_sub(2),
            (self.facts.len() as u16 + 2).min(area.height / 2),
        );
        let used = props::render(facts_area, buf, t, &self.facts, t.canvas);
        y += used + 1;
        let follow_h: u16 = if self.buttons.is_empty() { 0 } else { 2 };
        let body = Rect::new(
            area.x,
            y,
            area.width,
            area.bottom().saturating_sub(y + follow_h),
        );
        let focused = ctx.interaction.focused(BODY);
        let meta = format!("{} · y copies", plural(self.lines.len(), "line", "lines"));
        let panel = Panel::framed(Some(&self.detail_title))
            .focused(focused)
            .meta(&meta);
        let inner = panel.render(body, buf, t);
        self.scroll.set_content(self.lines.len());
        self.scroll.set_viewport(inner.height as usize);
        ctx.control(BODY, body, false);
        ctx.scrollable(BODY, inner);
        let has_sb = self.scroll.overflows();
        for (k, i) in self.scroll.visible_range().enumerate() {
            let (line, tone) = &self.lines[i];
            let st = ratatui::style::Style::new().fg(t.tone(*tone));
            buf.set_string(
                inner.x,
                inner.y + k as u16,
                truncate(line, inner.width.saturating_sub(u16::from(has_sb)) as usize),
                st,
            );
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                BODY,
                &self.scroll,
                focused,
            );
        }
        if follow_h > 0 {
            let fy = area.bottom().saturating_sub(1);
            buf.set_string(area.x + 1, fy, "Next", t.faint());
            let mut x = area.x + 7;
            for b in &mut self.buttons {
                let bw = b.width();
                if x + bw > area.right() {
                    break;
                }
                b.render(Rect::new(x, fy, bw, 1), buf, ctx, t.canvas);
                x += bw + 2;
            }
        }
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if focus.is_some_and(|f| self.buttons.iter().any(|b| b.id == f)) {
            return vec![
                hint("Enter", "Run"),
                hint("Tab", "Next"),
                hint("Esc", "Back"),
            ];
        }
        let mut v = vec![hint("↑↓", "Scroll"), hint("y", "Copy")];
        if !self.buttons.is_empty() {
            v.push(hint("Tab", "Next actions"));
        }
        v.push(hint("Esc", "Back"));
        v
    }

    fn crumb(&self, _w: &World) -> String {
        self.title.clone()
    }

    fn status(&self, _w: &World) -> StatusBits {
        StatusBits {
            center: Some(StatusItem::new("snapshot · read-only", Tone::Muted).priority(5)),
            right: vec![],
        }
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(BODY)
    }
}
