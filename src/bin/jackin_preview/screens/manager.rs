//! Host Workspace Manager: the current directory, saved Workspaces with
//! their instance children, `+ New workspace`, and a detail projection of
//! the selected row. Row identity is stable across background refreshes.

use std::collections::HashSet;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::{ButtonKind, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::layout::{Split, SplitDir};
use junie_tui::ui::text::{fit, truncate, truncate_middle, width};
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::picker::{Picker, PickerItem};
use junie_tui::widgets::progress::spinner_frame;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::segments::Segment;
use junie_tui::widgets::splitter::Splitter;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use super::modals::{InfoDialog, InfoResult};
use super::{Cx, Go, Modal, ModalResult, ModalTag, Screen, plural};
use crate::domain::agent::Agent;
use crate::domain::instance::{DaemonSnapshot, InstanceStatus};
use crate::domain::workspace::WorkspaceId;
use crate::sim::launch::LaunchPlan;
use crate::sim::world::{DaemonHealth, Msg, World};

pub const TREE: WidgetId = WidgetId::of("manager.tree");
pub const DETAIL: WidgetId = WidgetId::of("manager.detail");
pub const ROSTER: WidgetId = WidgetId::of("manager.roster");
pub const SEAM: WidgetId = WidgetId::of("manager.seam");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKey {
    CurrentDir,
    Workspace(WorkspaceId),
    Instance(String),
    NewWorkspace,
}

#[derive(Debug, Clone)]
struct Row {
    key: RowKey,
    depth: u16,
    glyph: &'static str,
    glyph_tone: Tone,
    label: String,
    meta: String,
    meta_tone: Tone,
    trailing: Option<(&'static str, Tone)>,
    expandable: bool,
}

/// Rows of the detail projection that can take the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DetailRow {
    Pane { tab: usize, pane: u64, text: String },
    Text(String, Tone),
    Session(String),
    Blank,
}

pub struct ManagerScreen {
    pub selected: RowKey,
    pub expanded: HashSet<WorkspaceId>,
    rows: Vec<Row>,
    pub scroll: ScrollState,
    pub detail_scroll: ScrollState,
    pub detail_cursor: usize,
    detail_rows: Vec<DetailRow>,
    pub split: Split,
    seam: Splitter,
    tree_area: Rect,
    detail_area: Rect,
    pub drawer_open: bool,
    actions: Vec<Button>,
    pub still_inside: Option<(String, Vec<String>, i64)>,
    pub busy_rows: Vec<(RowKey, &'static str)>,
    pending_launch: Option<(Option<WorkspaceId>, String)>,
    pending_session: Option<String>,
    picker_targets: Vec<(Agent, Option<String>)>,
    last_refresh_error: bool,
    body_narrow: bool,
    wide: bool,
    seam_container: Rect,
}

impl Default for ManagerScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagerScreen {
    pub fn new() -> Self {
        Self {
            selected: RowKey::CurrentDir,
            expanded: HashSet::new(),
            rows: vec![],
            scroll: ScrollState::default(),
            detail_scroll: ScrollState::default(),
            detail_cursor: 0,
            detail_rows: vec![],
            split: Split::new(32, 28, 40),
            seam: Splitter::new(SEAM, SplitDir::Horizontal),
            tree_area: Rect::ZERO,
            detail_area: Rect::ZERO,
            drawer_open: false,
            actions: vec![],
            still_inside: None,
            busy_rows: vec![],
            pending_launch: None,
            pending_session: None,
            picker_targets: vec![],
            last_refresh_error: false,
            body_narrow: false,
            wide: false,
            seam_container: Rect::ZERO,
        }
    }

    /// Select the instance row that was just left.
    pub fn select_instance(&mut self, id: &str, w: &World) {
        if let Some(i) = w.instance(id)
            && let Some(ws) = i.workspace
        {
            self.expanded.insert(ws);
        }
        self.selected = RowKey::Instance(id.to_owned());
    }

    fn build_rows(&mut self, w: &World) {
        let mut rows = vec![];
        let cwd_ws = w.cwd_workspace().map(|x| x.id);
        rows.push(Row {
            key: RowKey::CurrentDir,
            depth: 0,
            glyph: " ",
            glyph_tone: Tone::Normal,
            label: "Current directory".into(),
            meta: match cwd_ws.and_then(|id| w.workspace(id)) {
                Some(ws) => format!("saved as {}", ws.name),
                None => w.tilde(&w.cwd),
            },
            meta_tone: Tone::Muted,
            trailing: None,
            expandable: false,
        });
        for ws in &w.workspaces {
            let kids = w.instances_of(Some(ws.id));
            let running = kids.iter().filter(|i| i.status.is_live()).count();
            let failed = kids.iter().any(|i| {
                matches!(
                    i.status,
                    InstanceStatus::FailedSetup | InstanceStatus::Crashed
                )
            });
            let restore = kids
                .iter()
                .any(|i| i.status == InstanceStatus::RestoreAvailable);
            let (meta, tone) = if let Some((_, what)) = self
                .busy_rows
                .iter()
                .find(|(k, _)| *k == RowKey::Workspace(ws.id))
            {
                (
                    format!("{} {what}", spinner_frame(w.clock.now_ms as u64 / 80)),
                    Tone::Secondary,
                )
            } else if running > 0 {
                (format!("{running} running"), Tone::Secondary)
            } else if failed {
                ("! failed".into(), Tone::Error)
            } else if restore {
                ("restore available".into(), Tone::Secondary)
            } else if kids.is_empty() {
                ("idle".into(), Tone::Faint)
            } else {
                (plural(kids.len(), "record", "records"), Tone::Muted)
            };
            let expanded = self.expanded.contains(&ws.id);
            rows.push(Row {
                key: RowKey::Workspace(ws.id),
                depth: 0,
                glyph: if kids.is_empty() {
                    " "
                } else if expanded {
                    "▾"
                } else {
                    "▸"
                },
                glyph_tone: Tone::Secondary,
                label: ws.name.clone(),
                meta,
                meta_tone: tone,
                trailing: None,
                expandable: !kids.is_empty(),
            });
            if expanded {
                for i in kids {
                    let (glyph, gt) = match i.status {
                        InstanceStatus::Running => ("◉", Tone::Normal),
                        InstanceStatus::Crashed | InstanceStatus::FailedSetup => ("!", Tone::Error),
                        _ => ("◌", Tone::Muted),
                    };
                    let (meta, mt) = if let Some((_, what)) = self
                        .busy_rows
                        .iter()
                        .find(|(k, _)| *k == RowKey::Instance(i.id.clone()))
                    {
                        (
                            format!("{} {what}", spinner_frame(w.clock.now_ms as u64 / 80)),
                            Tone::Secondary,
                        )
                    } else {
                        match i.status {
                            InstanceStatus::Running => (
                                format!(
                                    "running {}",
                                    crate::clock::format_duration(
                                        (w.now_secs() - i.created_secs).max(0) as u64
                                    )
                                ),
                                Tone::Secondary,
                            ),
                            InstanceStatus::CleanExited => (
                                format!("exited clean · {}", w.clock.ago(i.last_seen_secs)),
                                Tone::Muted,
                            ),
                            InstanceStatus::Crashed => ("crashed · exit 137".into(), Tone::Error),
                            InstanceStatus::PreservedDirty => {
                                ("preserved · dirty".into(), Tone::Secondary)
                            }
                            InstanceStatus::PreservedUnpushed => {
                                ("preserved · unpushed".into(), Tone::Secondary)
                            }
                            InstanceStatus::RestoreAvailable => {
                                ("restore available".into(), Tone::Secondary)
                            }
                            InstanceStatus::FailedSetup => ("failed setup".into(), Tone::Error),
                            _ => (i.status.label().into(), Tone::Muted),
                        }
                    };
                    rows.push(Row {
                        key: RowKey::Instance(i.id.clone()),
                        depth: 1,
                        glyph,
                        glyph_tone: gt,
                        label: format!(
                            "{}  {} · {}",
                            i.id.trim_start_matches("jk-"),
                            i.role,
                            i.agent.label()
                        ),
                        meta,
                        meta_tone: mt,
                        trailing: if i.status.dirty() {
                            Some(("•", Tone::Warning))
                        } else {
                            None
                        },
                        expandable: false,
                    });
                }
            }
        }
        rows.push(Row {
            key: RowKey::NewWorkspace,
            depth: 0,
            glyph: " ",
            glyph_tone: Tone::Normal,
            label: "+ New workspace".into(),
            meta: String::new(),
            meta_tone: Tone::Muted,
            trailing: None,
            expandable: false,
        });
        self.rows = rows;
        // selection identity survives; a vanished row snaps to its parent
        if !self.rows.iter().any(|r| r.key == self.selected) {
            self.selected = match &self.selected {
                RowKey::Instance(id) => w
                    .instance(id)
                    .and_then(|i| i.workspace)
                    .map(RowKey::Workspace)
                    .unwrap_or(RowKey::CurrentDir),
                _ => RowKey::CurrentDir,
            };
        }
        self.scroll.set_content(self.rows.len());
        if let Some(i) = self.cursor() {
            self.scroll.ensure_visible(i);
        }
    }

    fn cursor(&self) -> Option<usize> {
        self.rows.iter().position(|r| r.key == self.selected)
    }

    fn move_cursor(&mut self, delta: isize) {
        let n = self.rows.len();
        if n == 0 {
            return;
        }
        let cur = self.cursor().unwrap_or(0) as isize;
        let next = (cur + delta).clamp(0, n as isize - 1) as usize;
        self.selected = self.rows[next].key.clone();
        self.scroll.ensure_visible(next);
        self.detail_cursor = 0;
        self.detail_scroll.jump_start();
    }

    fn selected_instance<'a>(&self, w: &'a World) -> Option<&'a crate::domain::instance::Instance> {
        match &self.selected {
            RowKey::Instance(id) => w.instance(id),
            _ => None,
        }
    }

    fn selected_workspace<'a>(
        &self,
        w: &'a World,
    ) -> Option<&'a crate::domain::workspace::Workspace> {
        match &self.selected {
            RowKey::Workspace(id) => w.workspace(*id),
            RowKey::CurrentDir => w.cwd_workspace(),
            _ => None,
        }
    }

    fn row_id(i: usize) -> WidgetId {
        TREE.child(i)
    }

    fn toggle_id(i: usize) -> WidgetId {
        TREE.child(i).sub("toggle")
    }

    fn detail_row_id(i: usize) -> WidgetId {
        DETAIL.child(i)
    }

    fn toggle_expand(&mut self, ws: WorkspaceId) {
        if !self.expanded.remove(&ws) {
            self.expanded.insert(ws);
        }
    }

    // ------------------------------------------------------------ actions

    fn open_launch_picker(&mut self, w: &World, cx: &mut Cx) {
        let (ws, role) = match &self.selected {
            RowKey::Workspace(id) => {
                let ws = w.workspace(*id);
                (
                    Some(*id),
                    ws.and_then(|x| x.roles.default.clone().or(x.roles.last.clone()))
                        .unwrap_or("the-architect".into()),
                )
            }
            RowKey::CurrentDir => (w.cwd_workspace().map(|x| x.id), "the-architect".into()),
            _ => return,
        };
        self.pending_launch = Some((ws, role.clone()));
        let mut p = Picker::new(WidgetId::of("manager.launch"), "Launch · choose Agent");
        p.searchable = false;
        p.width = 84;
        let scope = match ws.and_then(|id| w.workspace(id)) {
            Some(x) => format!("{} › {role}", x.name),
            None => format!("{} › {role}", w.tilde(&w.cwd)),
        };
        p.scope = Some(scope);
        let (items, targets) = agent_rows(w, ws, Some(&role));
        p.set_items(items);
        self.picker_targets = targets;
        cx.open(Modal::Picker(p), ModalTag::new("launch"));
    }

    fn open_session_picker(&mut self, instance: &str, w: &World, cx: &mut Cx) {
        let Some(i) = w.instance(instance) else {
            return;
        };
        self.pending_session = Some(instance.to_owned());
        let mut p = Picker::new(
            WidgetId::of("manager.session"),
            "New session · choose Agent",
        );
        p.searchable = false;
        p.width = 84;
        p.scope = Some(format!(
            "instance {} › {}",
            i.id.trim_start_matches("jk-"),
            i.role
        ));
        let (mut items, mut targets) = agent_rows(w, i.workspace, Some(&i.role));
        items.push(PickerItem {
            label: "Shell".into(),
            detail: "zsh · no provider account".into(),
            glyph: "$",
            group: "shells",
            tag: None,
            matched: vec![],
            disabled: false,
        });
        targets.push((Agent::ClaudeCode, Some("__shell__".into())));
        p.set_items(items);
        self.picker_targets = targets;
        cx.open(Modal::Picker(p), ModalTag::new("session"));
    }

    fn activate(&mut self, w: &mut World, cx: &mut Cx) -> Outcome {
        match self.selected.clone() {
            RowKey::CurrentDir | RowKey::Workspace(_) => {
                self.open_launch_picker(w, cx);
                Outcome::Changed
            }
            RowKey::NewWorkspace => {
                cx.go(Go::Prelude);
                Outcome::Changed
            }
            RowKey::Instance(id) => self.reconnect(&id, w, cx),
        }
    }

    fn reconnect(&mut self, id: &str, w: &mut World, cx: &mut Cx) -> Outcome {
        let Some(i) = w.instance(id) else {
            return Outcome::Consumed;
        };
        match i.status {
            InstanceStatus::Running => {
                cx.go(Go::Attach {
                    instance: id.to_owned(),
                    pane: None,
                });
            }
            InstanceStatus::RestoreAvailable
            | InstanceStatus::PreservedDirty
            | InstanceStatus::PreservedUnpushed => {
                // restore: the container restarts; a fresh daemon with one shell
                let wsname = i.workdir.trim_start_matches("/workspace/").to_owned();
                let agent = i.agent;
                let now_secs = w.clock.now_secs();
                if let Some(inst) = w.instance_mut(id) {
                    inst.status = InstanceStatus::Running;
                    inst.last_seen_secs = now_secs;
                }
                let now = w.now_ms();
                let mut d = crate::sim::pty::Daemon::new(&wsname);
                d.new_tab(Some(agent), None, now, false);
                w.daemons.insert(id.to_owned(), d);
                crate::domain::fixtures::refresh_snapshots(w);
                w.sync_arbiter();
                cx.status(format!(
                    "Restored {} · container restarted",
                    id.trim_start_matches("jk-")
                ));
                cx.go(Go::Attach {
                    instance: id.to_owned(),
                    pane: None,
                });
            }
            InstanceStatus::Crashed => {
                cx.error(format!(
                    "Cannot reconnect: {} crashed (exit 137) · inspect or purge it",
                    id.trim_start_matches("jk-")
                ));
            }
            InstanceStatus::FailedSetup => {
                cx.error(format!(
                    "Cannot reconnect: {} never reached the Capsule · launch again",
                    id.trim_start_matches("jk-")
                ));
            }
            InstanceStatus::CleanExited => {
                cx.status(format!(
                    "{} exited cleanly · launch the Workspace to start a new instance",
                    id.trim_start_matches("jk-")
                ));
            }
            _ => {}
        }
        Outcome::Changed
    }

    fn inspect(&mut self, id: &str, w: &World, cx: &mut Cx) {
        let Some(i) = w.instance(id) else { return };
        let ws = i.workspace.and_then(|x| w.workspace(x));
        let account = w.account_for(i.agent.provider(), ws, Some(&i.role), None);
        let mut props = vec![
            Prop::new("Container", i.container_id()).copyable(),
            Prop::new(
                "Image",
                format!(
                    "jackin/derived:{}-{}",
                    i.workdir.trim_start_matches("/workspace/"),
                    &i.run_id[4..8]
                ),
            ),
            Prop::new(
                "Workspace",
                format!(
                    "{} › role {}",
                    ws.map(|x| x.name.as_str()).unwrap_or("current directory"),
                    i.role
                ),
            ),
            Prop::new(
                "Agent",
                format!(
                    "{} · account {}",
                    i.agent.label(),
                    account.label(&w.accounts)
                ),
            ),
            Prop::new(
                "Target",
                format!(
                    "{} · {}",
                    i.workdir,
                    plural(ws.map(|x| x.mounts.len()).unwrap_or(1), "mount", "mounts")
                ),
            ),
            Prop::new("Run id", i.run_id.clone()).copyable(),
            Prop::new("Lifecycle", i.status.label()).tone(if i.status.is_live() {
                Tone::Normal
            } else {
                Tone::Secondary
            }),
        ];
        match &i.daemon {
            DaemonSnapshot::Unavailable => {
                props.push(Prop::new("Daemon", "unavailable").tone(Tone::Warning))
            }
            DaemonSnapshot::NoTabs => props.push(Prop::new("Daemon", "attached · no tabs")),
            DaemonSnapshot::Tabs(t) => props.push(Prop::new(
                "Daemon",
                format!(
                    "attached · {} · {}",
                    plural(t.len(), "tab", "tabs"),
                    w.clock.ago(i.last_seen_secs)
                ),
            )),
        }
        let d = InfoDialog::new(
            WidgetId::of("manager.info"),
            &format!("Container {}", id.trim_start_matches("jk-")),
            props,
        )
        .meta("read-only");
        cx.open(Modal::Info(d), ModalTag::new("info").key(id));
    }

    fn stop(&mut self, id: &str, w: &World, cx: &mut Cx) {
        let Some(i) = w.instance(id) else { return };
        if !i.status.stoppable() {
            cx.status(format!(
                "{} is not running · nothing to stop",
                id.trim_start_matches("jk-")
            ));
            return;
        }
        let short = id.trim_start_matches("jk-").to_owned();
        let d = Dialog::destructive(
            WidgetId::of("manager.stop"),
            &format!("Stop instance {short}?"),
            &format!(
                "The container stops and every session in it ends. The instance record stays reconnectable ({}).",
                if i.is_dirty() {
                    "preserved with uncommitted work"
                } else {
                    "restore available"
                }
            ),
            "Stop",
        );
        cx.open(Modal::Dialog(d), ModalTag::new("stop").key(id));
    }

    fn purge(&mut self, id: &str, w: &World, cx: &mut Cx) {
        let Some(i) = w.instance(id) else { return };
        let short = id.trim_start_matches("jk-").to_owned();
        let ws = i
            .workspace
            .and_then(|x| w.workspace(x))
            .map(|x| x.name.clone())
            .unwrap_or("current directory".into());
        let sessions = i.sessions.as_ref().map(|s| s.len()).unwrap_or(0);
        let facts = vec![
            Prop::new("Action", "purge the instance record and its container").tone(Tone::Error),
            Prop::new(
                "Target",
                format!("{short} · {ws} › {} · {}", i.role, i.agent.label()),
            ),
            Prop::new(
                "Scope",
                format!(
                    "1 container · {} · {}",
                    plural(sessions, "preserved session", "preserved sessions"),
                    if i.is_dirty() {
                        "worktree with changes"
                    } else {
                        "clean worktree"
                    }
                ),
            )
            .tone(Tone::Secondary),
            Prop::new(
                "Risk",
                if i.is_dirty() {
                    "uncommitted changes in the worktree are lost"
                } else {
                    "the container and its recovery state are removed"
                },
            )
            .tone(Tone::Warning),
            Prop::new("Reversible", "no").tone(Tone::Secondary),
        ];
        let d = Dialog::facts(
            WidgetId::of("manager.purge"),
            &format!("Purge instance {short}?"),
            facts,
            vec![],
            Some(&short),
            Button::danger(WidgetId::of("manager.purge").sub("ok"), "Purge"),
        );
        cx.open(Modal::Dialog(d), ModalTag::new("purge").key(id));
    }

    fn delete(&mut self, ws: WorkspaceId, w: &World, cx: &mut Cx) {
        let Some(x) = w.workspace(ws) else { return };
        let kids = w.instances_of(Some(ws)).len();
        let d = Dialog::destructive(
            WidgetId::of("manager.delete"),
            &format!("Delete workspace {}?", x.name),
            &format!(
                "The saved configuration is removed. {} and files on disk are kept.",
                if kids == 0 {
                    "Instances".to_owned()
                } else {
                    plural(kids, "instance record", "instance records")
                }
            ),
            "Delete",
        );
        cx.open(Modal::Dialog(d), ModalTag::new("delete").n(ws as usize));
    }

    fn prewarm(&mut self, ws: WorkspaceId, w: &mut World, cx: &mut Cx) {
        if self
            .busy_rows
            .iter()
            .any(|(k, _)| *k == RowKey::Workspace(ws))
        {
            cx.status("Prewarm already running");
            return;
        }
        self.busy_rows.push((RowKey::Workspace(ws), "prewarming…"));
        w.schedule(2_400, Msg::Prewarmed { workspace: ws });
        cx.status("Prewarming the derived image…");
    }

    fn open_github(&mut self, w: &World, cx: &mut Cx) {
        let Some(ws) = self.selected_workspace(w) else {
            cx.status("Select a saved workspace to open its repository");
            return;
        };
        let repo = w.github.iter().find(|r| r.full_name.ends_with(&ws.name));
        match repo {
            Some(r) => cx.status(format!("Opened {} on the host", r.url)),
            None => cx.status(format!("No GitHub source for {}", ws.name)),
        }
    }

    // -------------------------------------------------------------- detail

    fn build_detail(&mut self, w: &World) {
        let mut rows = vec![];
        if let Some(i) = self.selected_instance(w) {
            match &i.daemon {
                DaemonSnapshot::Tabs(tabs) => {
                    for (ti, t) in tabs.iter().enumerate() {
                        rows.push(DetailRow::Text(
                            format!(
                                "{} tab {}  {}",
                                if t.active { "▾" } else { "▸" },
                                ti + 1,
                                t.label
                            ),
                            Tone::Secondary,
                        ));
                        if t.active || tabs.len() <= 3 {
                            for (pi, p) in t.panes.iter().enumerate() {
                                let pane_id = w
                                    .daemons
                                    .get(&i.id)
                                    .and_then(|d| d.tabs.get(ti).map(|tt| tt.leaves()))
                                    .and_then(|l| l.get(pi).copied())
                                    .unwrap_or(0);
                                rows.push(DetailRow::Pane {
                                    tab: ti,
                                    pane: pane_id,
                                    text: format!(
                                        "{} pane {}  {:<10} {}",
                                        if p.focused { "›" } else { " " },
                                        pi + 1,
                                        p.agent.map(|a| a.short()).unwrap_or("shell"),
                                        p.state.label()
                                    ),
                                });
                            }
                        }
                    }
                }
                DaemonSnapshot::NoTabs => rows.push(DetailRow::Text(
                    "Daemon reports no tabs".into(),
                    Tone::Muted,
                )),
                DaemonSnapshot::Unavailable => rows.push(DetailRow::Text(
                    if i.status.is_live() {
                        "Daemon unavailable — showing manifest sessions".into()
                    } else {
                        "No live daemon (instance not running)".into()
                    },
                    Tone::Muted,
                )),
            }
            rows.push(DetailRow::Blank);
            match &i.sessions {
                Ok(s) if s.is_empty() => {
                    rows.push(DetailRow::Text("No sessions recorded".into(), Tone::Muted))
                }
                Ok(s) => {
                    for r in s {
                        rows.push(DetailRow::Session(format!(
                            "{:<5} {:<11} {:<8} {}",
                            r.id,
                            w.clock.ago(r.started_secs),
                            r.agent.map(|a| a.short()).unwrap_or("shell"),
                            r.status.label()
                        )));
                    }
                }
                Err(e) => rows.push(DetailRow::Text(format!("! {}", e.label()), Tone::Error)),
            }
        }
        self.detail_rows = rows;
        self.detail_cursor = self
            .detail_cursor
            .min(self.detail_rows.len().saturating_sub(1));
    }

    fn rebuild_actions(&mut self, w: &World) {
        let mut v = vec![];
        let mk =
            |name: &str, label: &str, kind: ButtonKind| Button::new(DETAIL.sub(name), label, kind);
        match &self.selected {
            RowKey::Instance(id) => {
                let Some(i) = w.instance(id) else {
                    self.actions = v;
                    return;
                };
                if i.status.is_live() {
                    v.push(mk("reconnect", "Reconnect", ButtonKind::Secondary));
                    v.push(mk("session", "New session", ButtonKind::Secondary));
                    v.push(mk("shell", "Shell", ButtonKind::Secondary));
                } else if i.status.reconnectable() && i.status != InstanceStatus::Crashed {
                    v.push(mk("reconnect", "Restore", ButtonKind::Secondary));
                }
                v.push(mk("inspect", "Inspect", ButtonKind::Secondary));
                if i.status.stoppable() {
                    v.push(mk("stop", "Stop", ButtonKind::Secondary));
                }
                v.push(mk("purge", "Purge…", ButtonKind::Danger));
            }
            RowKey::Workspace(_) => {
                v.push(mk("launch", "Launch", ButtonKind::Secondary));
                v.push(mk("edit", "Edit", ButtonKind::Secondary));
                v.push(mk("prewarm", "Prewarm", ButtonKind::Secondary));
                v.push(mk("delete", "Delete…", ButtonKind::Danger));
            }
            RowKey::CurrentDir => {
                v.push(mk("launch", "Launch", ButtonKind::Secondary));
                if w.cwd_workspace().is_some() {
                    v.push(mk("edit", "Edit", ButtonKind::Secondary));
                } else {
                    v.push(mk("new", "Create workspace", ButtonKind::Secondary));
                }
            }
            RowKey::NewWorkspace => v.push(mk("new", "Create workspace", ButtonKind::Secondary)),
        }
        self.actions = v;
    }

    fn fire_action(&mut self, name: &str, w: &mut World, cx: &mut Cx) -> Outcome {
        match (name, self.selected.clone()) {
            ("reconnect", RowKey::Instance(id)) => self.reconnect(&id, w, cx),
            ("session", RowKey::Instance(id)) => {
                self.open_session_picker(&id, w, cx);
                Outcome::Changed
            }
            ("shell", RowKey::Instance(id)) => {
                cx.go(Go::NewSession {
                    instance: id,
                    agent: None,
                    account: None,
                });
                Outcome::Changed
            }
            ("inspect", RowKey::Instance(id)) => {
                self.inspect(&id, w, cx);
                Outcome::Changed
            }
            ("stop", RowKey::Instance(id)) => {
                self.stop(&id, w, cx);
                Outcome::Changed
            }
            ("purge", RowKey::Instance(id)) => {
                self.purge(&id, w, cx);
                Outcome::Changed
            }
            ("launch", _) => {
                self.open_launch_picker(w, cx);
                Outcome::Changed
            }
            ("edit", RowKey::Workspace(id)) => {
                cx.go(Go::Editor {
                    workspace: Some(id),
                    pending: None,
                });
                Outcome::Changed
            }
            ("edit", RowKey::CurrentDir) => {
                if let Some(ws) = w.cwd_workspace() {
                    cx.go(Go::Editor {
                        workspace: Some(ws.id),
                        pending: None,
                    });
                }
                Outcome::Changed
            }
            ("prewarm", RowKey::Workspace(id)) => {
                self.prewarm(id, w, cx);
                Outcome::Changed
            }
            ("delete", RowKey::Workspace(id)) => {
                self.delete(id, w, cx);
                Outcome::Changed
            }
            ("new", _) => {
                cx.go(Go::Prelude);
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    fn drawer_visible(&self, focus: Option<WidgetId>) -> bool {
        self.body_narrow
            && (self.drawer_open
                || focus.is_some_and(|f| f == DETAIL || self.actions.iter().any(|b| b.id == f)))
    }

    // ------------------------------------------------------------- render

    fn draw_tree(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(TREE);
        let running = w.running_count();
        let pos = scrollbar::position_label(&self.scroll);
        let meta = if pos.is_empty() {
            if running > 0 {
                format!("{running} running")
            } else {
                "no instances".into()
            }
        } else if running > 0 {
            format!("{running} running · {pos}")
        } else {
            pos
        };
        let inner = Panel::framed(Some("Workspaces"))
            .focused(focused)
            .meta(&meta)
            .render(area, buf, t);
        self.tree_area = inner;
        let bg = t.canvas;
        self.scroll.set_viewport(inner.height as usize);
        if let Some(i) = self.cursor() {
            self.scroll.ensure_visible(i);
        }
        ctx.control(TREE, inner, false);
        ctx.scrollable(TREE, inner);
        let has_sb = self.scroll.overflows();
        let row_w = inner.width.saturating_sub(u16::from(has_sb));
        let cursor = self.cursor();
        // metadata is all-or-none for the pane
        let show_meta = row_w >= 44;
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = inner.y + k as u16;
            let row = &self.rows[i];
            let rid = Self::row_id(i);
            let mut s = ctx.state(rid);
            if ctx.interaction.hovered(Self::toggle_id(i)) {
                s.hovered = true;
            }
            s.focused = focused && cursor == Some(i);
            s.selected = cursor == Some(i);
            let st = t.row(s, bg);
            let rect = Rect::new(inner.x - 1, y, row_w + 1, 1);
            fill(buf, rect, st);
            buf.set_string(rect.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            let mut x = rect.x + 2 + row.depth * 2;
            let gs = st
                .fg(t.tone(row.glyph_tone))
                .remove_modifier(Modifier::BOLD);
            let gs = if s.focused && row.glyph_tone == Tone::Normal {
                st
            } else {
                gs
            };
            buf.set_string(x, y, row.glyph, gs);
            if row.expandable {
                ctx.clickable(Self::toggle_id(i), Rect::new(x, y, 2, 1));
            }
            x += 2;
            let meta_w = if show_meta {
                width(&row.meta) as u16
            } else {
                0
            };
            let trailing_w: u16 = if row.trailing.is_some() { 2 } else { 0 };
            let avail = rect.right().saturating_sub(x + 1);
            let lw = avail.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 0 } + trailing_w);
            let label_style = if row.key == RowKey::NewWorkspace {
                st.fg(if s.focused {
                    t.text_primary
                } else {
                    t.text_secondary
                })
            } else if s.selected {
                st.fg(t.accent)
            } else {
                st
            };
            buf.set_string(
                x,
                y,
                fit(&truncate_middle(&row.label, lw as usize), lw as usize),
                label_style,
            );
            if meta_w > 0 && meta_w + 4 < avail {
                buf.set_string(
                    rect.right().saturating_sub(meta_w + 1 + trailing_w),
                    y,
                    &row.meta,
                    st.fg(t.tone(row.meta_tone)).remove_modifier(Modifier::BOLD),
                );
            }
            if let Some((g, tone)) = row.trailing {
                buf.set_string(rect.right().saturating_sub(2), y, g, st.fg(t.tone(tone)));
            }
            ctx.clickable(rid, rect);
            if row.expandable {
                ctx.clickable(
                    Self::toggle_id(i),
                    Rect::new(rect.x + 2 + row.depth * 2, y, 2, 1),
                );
            }
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                TREE,
                &self.scroll,
                focused,
            );
        }
    }

    fn draw_detail(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        w: &World,
        as_drawer: bool,
    ) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(DETAIL)
            || self.actions.iter().any(|b| ctx.interaction.focused(b.id));
        self.build_detail(w);
        self.rebuild_actions(w);
        let (title, scope_word): (String, String) = match &self.selected {
            RowKey::CurrentDir => (
                match w.cwd_workspace() {
                    Some(ws) => format!("Current directory · {}", ws.name),
                    None => "Current directory".into(),
                },
                match w.cwd_workspace() {
                    Some(_) => "saved workspace".into(),
                    None => "not saved".into(),
                },
            ),
            RowKey::Workspace(id) => {
                let ws = w.workspace(*id);
                let n = w
                    .instances_of(Some(*id))
                    .iter()
                    .filter(|i| i.status.is_live())
                    .count();
                (
                    ws.map(|x| x.name.clone()).unwrap_or_default(),
                    if n > 0 {
                        format!("saved workspace · {n} running")
                    } else {
                        "saved workspace".into()
                    },
                )
            }
            RowKey::Instance(id) => {
                let i = w.instance(id);
                (
                    format!(
                        "{} · {}",
                        id.trim_start_matches("jk-"),
                        i.and_then(|x| x.workspace)
                            .and_then(|x| w.workspace(x))
                            .map(|x| x.name.as_str())
                            .unwrap_or("current directory")
                    ),
                    format!("instance · {}", i.map(|x| x.status.label()).unwrap_or("?")),
                )
            }
            RowKey::NewWorkspace => ("New workspace".into(), "create".into()),
        };
        let inner = if as_drawer {
            Panel::framed(Some(&title))
                .focused(focused)
                .meta(&scope_word)
                .render(area, buf, t)
        } else {
            Panel::card(Some(&title))
                .focused(focused)
                .meta(&scope_word)
                .render(area, buf, t)
        };
        self.detail_area = inner;
        let bg = if as_drawer { t.canvas } else { t.surface };
        ctx.scrollable(DETAIL, inner);
        let mut y = inner.y;
        let put = |buf: &mut Buffer, y: u16, text: &str, style: Style| {
            if y < inner.bottom() {
                buf.set_string(
                    inner.x,
                    y,
                    truncate(text, inner.width as usize),
                    style.bg(bg),
                );
            }
        };
        match self.selected.clone() {
            RowKey::CurrentDir | RowKey::Workspace(_) => {
                let ws = self.selected_workspace(w);
                match ws {
                    Some(ws) => {
                        let mut facts = vec![
                            Prop::new("Working dir", ws.workdir.clone()),
                            Prop::new(
                                "Mounts",
                                if ws.mounts.is_empty() {
                                    "none".into()
                                } else {
                                    ws.mounts
                                        .iter()
                                        .map(|m| {
                                            format!(
                                                "{} · {} {}",
                                                w.tilde(m.source_label()),
                                                m.mode_label(),
                                                m.isolation.label().to_lowercase()
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                },
                            )
                            .wrap(),
                            Prop::new(
                                "Roles",
                                format!(
                                    "{}{}",
                                    ws.roles
                                        .default
                                        .as_ref()
                                        .map(|d| format!("{d} ★ · "))
                                        .unwrap_or_default(),
                                    match &ws.roles.allowed {
                                        crate::domain::workspace::AllowedRoles::All =>
                                            format!("allowed all ({} in registry)", w.roles.len()),
                                        crate::domain::workspace::AllowedRoles::Custom(l) =>
                                            format!("allowed {} of {}", l.len(), w.roles.len()),
                                    }
                                ),
                            ),
                            Prop::new(
                                "Environments",
                                format!(
                                    "{} · {} [op]",
                                    plural(ws.env_count(), "var", "vars"),
                                    ws.env
                                        .iter()
                                        .chain(ws.role_env.values().flatten())
                                        .filter(|e| matches!(
                                            e.value,
                                            crate::domain::workspace::EnvValue::OnePassword(_)
                                        ))
                                        .count()
                                ),
                            ),
                        ];
                        let mut auth_lines = vec![];
                        for a in Agent::ALL {
                            let r = w.account_for(
                                a.provider(),
                                Some(ws),
                                ws.roles.default.as_deref(),
                                None,
                            );
                            if r.account.is_some()
                                && (ws.account_overrides.contains_key(&a.provider())
                                    || ws.auth.iter().any(|x| x.agent == a))
                            {
                                auth_lines.push(format!(
                                    "{} · {} ({})",
                                    a.label(),
                                    r.label(&w.accounts),
                                    r.level.label()
                                ));
                            }
                        }
                        if auth_lines.is_empty() {
                            auth_lines.push("provider defaults".into());
                        }
                        facts.push(Prop::new("Auth", auth_lines.join("\n")).wrap());
                        facts.push(Prop::new(
                            "Policies",
                            format!(
                                "git pull {} · keep awake {} · dirty exit {}",
                                if ws.git_pull { "enabled" } else { "disabled" },
                                if ws.keep_awake { "on" } else { "off" },
                                ws.dirty_policy.label()
                            ),
                        ));
                        let used = props::render(
                            Rect::new(inner.x, y, inner.width, inner.height),
                            buf,
                            t,
                            &facts,
                            bg,
                        );
                        y += used + 1;
                        let kids = w.instances_of(Some(ws.id));
                        if !kids.is_empty() {
                            put(
                                buf,
                                y,
                                "Instances",
                                t.secondary().add_modifier(Modifier::BOLD),
                            );
                            let meta = match w.daemon_health {
                                DaemonHealth::Healthy => {
                                    format!("daemon · {}", w.clock.ago(w.last_refresh_secs))
                                }
                                DaemonHealth::Stale => {
                                    format!("▲ daemon stale · {}", w.clock.ago(w.last_refresh_secs))
                                }
                            };
                            let mw = width(&meta) as u16;
                            if y < inner.bottom() && inner.width > mw + 12 {
                                buf.set_string(inner.right() - mw, y, &meta, t.faint().bg(bg));
                            }
                            y += 1;
                            for i in kids {
                                let line = format!(
                                    "{} {}  {} · {} · {}",
                                    match i.status {
                                        InstanceStatus::Running => "◉",
                                        InstanceStatus::Crashed | InstanceStatus::FailedSetup =>
                                            "!",
                                        _ => "◌",
                                    },
                                    i.id.trim_start_matches("jk-"),
                                    i.role,
                                    i.agent.label(),
                                    i.status.label()
                                );
                                put(
                                    buf,
                                    y,
                                    &line,
                                    if i.status.is_live() {
                                        t.primary()
                                    } else {
                                        t.secondary()
                                    },
                                );
                                y += 1;
                            }
                        }
                    }
                    None => {
                        let e = EmptyState::new("Create a workspace from this directory.").hint(&format!(
                            "{} is mounted at its own path inside the Construct. Enter launches with defaults; n creates a saved workspace.",
                            w.tilde(&w.cwd)
                        ));
                        empty::render(
                            Rect::new(
                                inner.x,
                                inner.y,
                                inner.width,
                                inner.height.saturating_sub(3),
                            ),
                            buf,
                            t,
                            &e,
                            bg,
                        );
                        y = inner.bottom().saturating_sub(2);
                    }
                }
            }
            RowKey::NewWorkspace => {
                let e = EmptyState::new("New workspace").hint("Enter starts the five-step create chain: source, destination, working directory, name, then the editor.");
                empty::render(
                    Rect::new(
                        inner.x,
                        inner.y,
                        inner.width,
                        inner.height.saturating_sub(3),
                    ),
                    buf,
                    t,
                    &e,
                    bg,
                );
                y = inner.bottom().saturating_sub(2);
            }
            RowKey::Instance(id) => {
                let Some(i) = w.instance(&id) else { return };
                let ws = i.workspace.and_then(|x| w.workspace(x));
                let acc = w.account_for(i.agent.provider(), ws, Some(&i.role), None);
                let facts = vec![
                    Prop::new(
                        "Workspace",
                        format!(
                            "{} › role {}",
                            ws.map(|x| x.name.as_str()).unwrap_or("current directory"),
                            i.role
                        ),
                    ),
                    Prop::new(
                        "Agent",
                        format!(
                            "{} · account {} ({})",
                            i.agent.label(),
                            acc.label(&w.accounts),
                            acc.level.label()
                        ),
                    ),
                    Prop::new("Container", i.container_id()),
                    Prop::new(
                        "Started",
                        format!(
                            "{} · last seen {}",
                            w.clock.ago(i.created_secs),
                            w.clock.ago(i.last_seen_secs)
                        ),
                    ),
                    Prop::new(
                        "Lifecycle",
                        format!("{} · {}", i.status.label(), i.status.description()),
                    )
                    .tone(match i.status {
                        InstanceStatus::Crashed | InstanceStatus::FailedSetup => Tone::Error,
                        InstanceStatus::Running => Tone::Normal,
                        _ => Tone::Secondary,
                    })
                    .wrap(),
                    Prop::new("Working tree", i.dirty_summary()).tone(if i.is_dirty() {
                        Tone::Warning
                    } else {
                        Tone::Secondary
                    }),
                ];
                let mut facts = facts;
                if let Some(b) = &i.branch {
                    facts.push(Prop::new(
                        "Branch",
                        match &i.pr {
                            Some((n, title)) => format!("{b} · PR #{n} · {title}"),
                            None => b.clone(),
                        },
                    ));
                }
                let used = props::render(
                    Rect::new(inner.x, y, inner.width, inner.height),
                    buf,
                    t,
                    &facts,
                    bg,
                );
                y += used + 1;
                let live_focus = ctx.interaction.focused(DETAIL);
                put(
                    buf,
                    y,
                    "Live topology",
                    t.secondary().add_modifier(Modifier::BOLD),
                );
                let src = match &i.daemon {
                    DaemonSnapshot::Unavailable => "unavailable".to_owned(),
                    _ => format!("daemon · {}", w.clock.ago(i.last_seen_secs)),
                };
                let sw = width(&src) as u16;
                if y < inner.bottom() && inner.width > sw + 16 {
                    buf.set_string(inner.right() - sw, y, &src, t.faint().bg(bg));
                }
                y += 1;
                let list_top = y;
                let rows_avail = inner.bottom().saturating_sub(y + 2) as usize;
                self.detail_scroll.set_content(self.detail_rows.len());
                self.detail_scroll.set_viewport(rows_avail);
                if live_focus {
                    self.detail_scroll.ensure_visible(self.detail_cursor);
                }
                ctx.control(
                    DETAIL,
                    Rect::new(inner.x, list_top, inner.width, rows_avail as u16),
                    false,
                );
                let mut sessions_heading_done = false;
                for (k, ri) in self.detail_scroll.visible_range().enumerate() {
                    let yy = list_top + k as u16;
                    let row = &self.detail_rows[ri];
                    let rid = Self::detail_row_id(ri);
                    match row {
                        DetailRow::Pane { text, .. } => {
                            let mut s = ctx.state(rid);
                            s.focused = live_focus && ri == self.detail_cursor;
                            let st = t.row(s, bg);
                            let r = Rect::new(inner.x - 1, yy, inner.width + 1, 1);
                            fill(buf, r, st);
                            buf.set_string(r.x, yy, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
                            buf.set_string(
                                r.x + 2,
                                yy,
                                truncate(text, r.width.saturating_sub(3) as usize),
                                st,
                            );
                            ctx.clickable(rid, r);
                        }
                        DetailRow::Text(text, tone) => {
                            put(buf, yy, text, Style::new().fg(t.tone(*tone)))
                        }
                        DetailRow::Session(text) => {
                            if !sessions_heading_done {
                                sessions_heading_done = true;
                            }
                            put(buf, yy, text, t.secondary());
                        }
                        DetailRow::Blank => {
                            put(
                                buf,
                                yy,
                                "Sessions",
                                t.secondary().add_modifier(Modifier::BOLD),
                            );
                            let m = format!(
                                "manifest · {}",
                                plural(
                                    i.sessions.as_ref().map(|s| s.len()).unwrap_or(0),
                                    "recorded",
                                    "recorded"
                                )
                            );
                            let mw = width(&m) as u16;
                            if yy < inner.bottom() && inner.width > mw + 12 {
                                buf.set_string(inner.right() - mw, yy, &m, t.faint().bg(bg));
                            }
                        }
                    }
                }
                if self.detail_scroll.overflows() {
                    scrollbar::render_vertical(
                        Rect::new(inner.right() - 1, list_top, 1, rows_avail as u16),
                        buf,
                        ctx,
                        DETAIL,
                        &self.detail_scroll,
                        live_focus,
                    );
                }
                y = inner.bottom().saturating_sub(2);
            }
        }
        // action row at the bottom of the card
        let ay = inner.bottom().saturating_sub(1).max(y);
        if ay < inner.bottom() {
            let widths: Vec<u16> = self.actions.iter().map(|b| b.width()).collect();
            let rects = row_layout(Rect::new(inner.x - 1, ay, inner.width + 1, 1), &widths, 2);
            for (b, r) in self.actions.iter_mut().zip(rects) {
                b.render(r, buf, ctx, bg);
            }
        }
        if !matches!(self.selected, RowKey::Instance(_)) {
            // the card itself is the stop when there are no pane rows
            ctx.control(DETAIL, Rect::new(inner.x, inner.y, inner.width, 1), false);
        }
    }

    fn draw_roster(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(ROSTER);
        let running = w.running();
        let inner = Panel::card(Some("Running"))
            .focused(focused)
            .meta(&plural(running.len(), "instance", "instances"))
            .render(area, buf, t);
        let bg = t.surface;
        ctx.control(ROSTER, inner, false);
        let mut y = inner.y;
        for i in &running {
            if y + 2 >= inner.bottom() {
                break;
            }
            let ws = i
                .workspace
                .and_then(|x| w.workspace(x))
                .map(|x| x.name.as_str())
                .unwrap_or("current directory");
            let acc = w.account_for(
                i.agent.provider(),
                i.workspace.and_then(|x| w.workspace(x)),
                Some(&i.role),
                None,
            );
            let rid = ROSTER.child(y as usize);
            let mut s = ctx.state(rid);
            s.selected = self.selected == RowKey::Instance(i.id.clone());
            let st = t.row(s, bg);
            let r = Rect::new(inner.x - 1, y, inner.width + 1, 3);
            fill(buf, r, st);
            buf.set_string(
                inner.x,
                y,
                truncate(
                    &format!(
                        "◉ {}  {ws} · {} · {}",
                        i.id.trim_start_matches("jk-"),
                        i.role,
                        i.agent.label()
                    ),
                    inner.width as usize,
                ),
                st,
            );
            let live = match &i.daemon {
                DaemonSnapshot::Tabs(tabs) => format!(
                    "running {} · {} · {}",
                    crate::clock::format_duration((w.now_secs() - i.created_secs).max(0) as u64),
                    plural(tabs.len(), "tab", "tabs"),
                    plural(tabs.iter().map(|t| t.panes.len()).sum(), "pane", "panes")
                ),
                _ => format!(
                    "running {} · daemon unavailable",
                    crate::clock::format_duration((w.now_secs() - i.created_secs).max(0) as u64)
                ),
            };
            buf.set_string(
                inner.x + 3,
                y + 1,
                truncate(&live, inner.width.saturating_sub(3) as usize),
                st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
            );
            buf.set_string(
                inner.x + 3,
                y + 2,
                truncate(
                    &format!("account {}", acc.label(&w.accounts)),
                    inner.width.saturating_sub(3) as usize,
                ),
                st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
            );
            ctx.clickable(rid, r);
            y += 4;
        }
        let preserved: Vec<_> = w
            .instances
            .iter()
            .filter(|i| i.status.dirty() || i.status == InstanceStatus::RestoreAvailable)
            .collect();
        if !preserved.is_empty() && y + 2 < inner.bottom() {
            y += 1;
            buf.set_string(
                inner.x,
                y,
                "Preserved",
                t.secondary().bg(bg).add_modifier(Modifier::BOLD),
            );
            let m = plural(preserved.len(), "record", "records");
            buf.set_string(inner.right() - width(&m) as u16, y, &m, t.faint().bg(bg));
            y += 1;
            for i in preserved {
                if y >= inner.bottom() {
                    break;
                }
                let ws = i
                    .workspace
                    .and_then(|x| w.workspace(x))
                    .map(|x| x.name.as_str())
                    .unwrap_or("current directory");
                buf.set_string(
                    inner.x,
                    y,
                    truncate(
                        &format!(
                            "◌ {}  {ws} · {} · {}",
                            i.id.trim_start_matches("jk-"),
                            i.role,
                            i.status.label()
                        ),
                        inner.width as usize,
                    ),
                    t.secondary().bg(bg),
                );
                y += 1;
            }
        }
        if y + 3 < inner.bottom() {
            y += 1;
            buf.set_string(
                inner.x,
                y,
                "Daemon",
                t.secondary().bg(bg).add_modifier(Modifier::BOLD),
            );
            let (h, tone) = match w.daemon_health {
                DaemonHealth::Healthy => (
                    format!("healthy · {}", w.clock.ago(w.last_refresh_secs)),
                    Tone::Secondary,
                ),
                DaemonHealth::Stale => (
                    format!("▲ stale · {}", w.clock.ago(w.last_refresh_secs)),
                    Tone::Warning,
                ),
            };
            buf.set_string(
                inner.right() - width(&h) as u16,
                y,
                &h,
                Style::new().fg(t.tone(tone)).bg(bg),
            );
            y += 1;
            buf.set_string(
                inner.x,
                y,
                "Refresh      throttled · every 5 s",
                t.muted().bg(bg),
            );
            y += 1;
            let summary = crate::domain::usage::OverallSummary::compute(&w.accounts.accounts);
            buf.set_string(
                inner.x,
                y,
                truncate(
                    &format!(
                        "Usage        {} · {}",
                        summary.health.label(),
                        summary.issues_line()
                    ),
                    inner.width as usize,
                ),
                t.muted().bg(bg),
            );
        }
    }
}

/// Picker rows for every Agent with its resolved account and why.
fn agent_rows(
    w: &World,
    ws: Option<WorkspaceId>,
    role: Option<&str>,
) -> (Vec<PickerItem>, Vec<(Agent, Option<String>)>) {
    let wsr = ws.and_then(|id| w.workspace(id));
    let mut items = vec![];
    let mut targets = vec![];
    for a in Agent::ALL {
        let r = w.account_for(a.provider(), wsr, role, None);
        let (detail, disabled) = match &r.account {
            Some(id) => {
                let acc = w.accounts.get(id);
                let health = acc.map(|x| x.status_word()).unwrap_or("?");
                (
                    format!(
                        "account {} · {} · {}",
                        acc.map(|x| x.title()).unwrap_or(id.clone()),
                        r.level.label(),
                        health
                    ),
                    false,
                )
            }
            None => (
                "no account · register one in Accounts (c) or launch with the host profile".into(),
                false,
            ),
        };
        items.push(PickerItem {
            label: a.label().into(),
            detail,
            glyph: if a.registerable() { "▪" } else { "·" },
            group: "agents",
            tag: if r.account.is_some() {
                Some(r.level.label())
            } else {
                Some("host profile")
            },
            matched: vec![],
            disabled,
        });
        targets.push((a, r.account.clone()));
    }
    (items, targets)
}

impl Screen for ManagerScreen {
    fn enter(&mut self, w: &mut World, cx: &mut Cx) {
        self.build_rows(w);
        cx.focus.focus(TREE);
        self.drawer_open = false;
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(TREE)
    }

    fn on_tick(&mut self, w: &mut World, _cx: &mut Cx) -> Outcome {
        let mut out = Outcome::Ignored;
        if let Some((_, _, until)) = &self.still_inside
            && w.now_ms() >= *until
        {
            self.still_inside = None;
            out = Outcome::Changed;
        }
        if !self.busy_rows.is_empty() {
            out = Outcome::Changed;
        }
        // throttled refresh every 5 s of virtual time
        if w.now_secs() - w.last_refresh_secs >= 5 {
            w.last_refresh_secs = w.now_secs();
            if w.refresh_fails && !self.last_refresh_error {
                self.last_refresh_error = true;
                w.daemon_health = DaemonHealth::Stale;
                w.schedule(0, Msg::Refreshed { ok: false });
            } else {
                crate::domain::fixtures::refresh_snapshots(w);
                for i in w.instances.iter_mut() {
                    if i.status.is_live() {
                        i.last_seen_secs = w.clock.now_secs();
                    }
                }
            }
            out = Outcome::Changed;
        }
        self.build_rows(w);
        out
    }

    fn on_msg(&mut self, msg: &Msg, w: &mut World, cx: &mut Cx) -> Outcome {
        match msg {
            Msg::Prewarmed { workspace } => {
                self.busy_rows
                    .retain(|(k, _)| *k != RowKey::Workspace(*workspace));
                let name = w
                    .workspace(*workspace)
                    .map(|x| x.name.clone())
                    .unwrap_or_default();
                cx.status(format!("Prewarmed {name} · derived image cached"));
            }
            Msg::Stopped { instance } => {
                self.busy_rows
                    .retain(|(k, _)| *k != RowKey::Instance(instance.clone()));
                let dirty = w.instance(instance).is_some_and(|i| i.is_dirty());
                let now_secs = w.clock.now_secs();
                if let Some(i) = w.instance_mut(instance) {
                    i.status = if dirty {
                        InstanceStatus::PreservedDirty
                    } else {
                        InstanceStatus::RestoreAvailable
                    };
                    i.last_seen_secs = now_secs;
                }
                w.daemons.remove(instance);
                crate::domain::fixtures::refresh_snapshots(w);
                w.sync_arbiter();
                cx.status(format!(
                    "Instance {} stopped",
                    instance.trim_start_matches("jk-")
                ));
            }
            Msg::Purged { instance } => {
                self.busy_rows
                    .retain(|(k, _)| *k != RowKey::Instance(instance.clone()));
                if let Some(i) = w.instance_mut(instance) {
                    i.status = InstanceStatus::Purged;
                }
                w.daemons.remove(instance);
                w.sync_arbiter();
                cx.status(format!(
                    "Purged {} · container, sidecar, volume and recovery state removed",
                    instance.trim_start_matches("jk-")
                ));
            }
            Msg::Refreshed { ok: false } => {
                cx.error("Refresh failed: instance index unreadable · showing last-good rows");
            }
            _ => return Outcome::Ignored,
        }
        self.build_rows(w);
        Outcome::Changed
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        self.build_rows(w);
        let focus = cx.focus.current();
        let in_tree = focus == Some(TREE);
        let in_detail = focus == Some(DETAIL);
        let action_focus = self.actions.iter().position(|b| Some(b.id) == focus);
        // global letters (any focus)
        match key.code {
            KeyCode::Char('n') if key.plain() && action_focus.is_none() => {
                if let RowKey::Instance(id) = self.selected.clone() {
                    self.open_session_picker(&id, w, cx);
                } else {
                    cx.go(Go::Prelude);
                }
                return Outcome::Changed;
            }
            KeyCode::Char('e') if key.plain() => {
                return match self.selected.clone() {
                    RowKey::Workspace(id) => {
                        cx.go(Go::Editor {
                            workspace: Some(id),
                            pending: None,
                        });
                        Outcome::Changed
                    }
                    RowKey::CurrentDir => {
                        match w.cwd_workspace() {
                            Some(ws) => {
                                let id = ws.id;
                                cx.go(Go::Editor {
                                    workspace: Some(id),
                                    pending: None,
                                });
                                Outcome::Changed
                            }
                            None => {
                                cx.status("The current directory is not a saved workspace · n creates one");
                                Outcome::Changed
                            }
                        }
                    }
                    _ => Outcome::Ignored,
                };
            }
            KeyCode::Char('d') if key.plain() => {
                if let RowKey::Workspace(id) = self.selected {
                    self.delete(id, w, cx);
                    return Outcome::Changed;
                }
            }
            KeyCode::Char('w') if key.plain() => {
                if let RowKey::Workspace(id) = self.selected {
                    self.prewarm(id, w, cx);
                    return Outcome::Changed;
                }
            }
            KeyCode::Char('o') if key.plain() => {
                self.open_github(w, cx);
                return Outcome::Changed;
            }
            KeyCode::Char('r') if key.plain() => {
                if let RowKey::Instance(id) = self.selected.clone() {
                    return self.reconnect(&id, w, cx);
                }
            }
            KeyCode::Char('a') if key.plain() => {
                if let RowKey::Instance(id) = self.selected.clone() {
                    self.open_session_picker(&id, w, cx);
                    return Outcome::Changed;
                }
            }
            KeyCode::Char('x') if key.plain() => {
                if let RowKey::Instance(id) = self.selected.clone() {
                    if w.instance(&id).is_some_and(|i| i.status.is_live()) {
                        cx.go(Go::NewSession {
                            instance: id,
                            agent: None,
                            account: None,
                        });
                    } else {
                        cx.status("Shell needs a running instance · reconnect or restore first");
                    }
                    return Outcome::Changed;
                }
            }
            KeyCode::Char('i') if key.plain() => {
                if let RowKey::Instance(id) = self.selected.clone() {
                    self.inspect(&id, w, cx);
                    return Outcome::Changed;
                }
            }
            KeyCode::Char('t') if key.plain() => {
                if let RowKey::Instance(id) = self.selected.clone() {
                    self.stop(&id, w, cx);
                    return Outcome::Changed;
                }
            }
            KeyCode::Char('p') if key.plain() => {
                if let RowKey::Instance(id) = self.selected.clone() {
                    self.purge(&id, w, cx);
                    return Outcome::Changed;
                }
            }
            KeyCode::F(5) => {
                w.last_refresh_secs = w.now_secs() - 6;
                cx.status("Refreshing…");
                return Outcome::Changed;
            }
            _ => {}
        }
        if in_tree {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_cursor(-1);
                    return Outcome::Changed;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_cursor(1);
                    return Outcome::Changed;
                }
                KeyCode::PageUp => {
                    self.move_cursor(-(self.scroll.viewport_len.max(1) as isize));
                    return Outcome::Changed;
                }
                KeyCode::PageDown => {
                    self.move_cursor(self.scroll.viewport_len.max(1) as isize);
                    return Outcome::Changed;
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    self.move_cursor(-(self.rows.len() as isize));
                    return Outcome::Changed;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    self.move_cursor(self.rows.len() as isize);
                    return Outcome::Changed;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if let RowKey::Workspace(id) = self.selected {
                        if self.expanded.contains(&id) {
                            self.move_cursor(1);
                        } else if !w.instances_of(Some(id)).is_empty() {
                            self.expanded.insert(id);
                        }
                    }
                    self.build_rows(w);
                    return Outcome::Changed;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    match self.selected.clone() {
                        RowKey::Workspace(id) if self.expanded.contains(&id) => {
                            self.expanded.remove(&id);
                        }
                        RowKey::Instance(iid) => {
                            if let Some(ws) = w.instance(&iid).and_then(|i| i.workspace) {
                                self.selected = RowKey::Workspace(ws);
                            }
                        }
                        _ => {}
                    }
                    self.build_rows(w);
                    return Outcome::Changed;
                }
                KeyCode::Char(' ') => {
                    if let RowKey::Workspace(id) = self.selected {
                        self.toggle_expand(id);
                        self.build_rows(w);
                    }
                    return Outcome::Changed;
                }
                KeyCode::Char('*') => {
                    for ws in &w.workspaces {
                        self.expanded.insert(ws.id);
                    }
                    self.build_rows(w);
                    return Outcome::Changed;
                }
                KeyCode::Char('-') => {
                    self.expanded.clear();
                    self.build_rows(w);
                    return Outcome::Changed;
                }
                KeyCode::Enter => return self.activate(w, cx),
                KeyCode::Tab => {
                    self.drawer_open = true;
                    cx.focus.focus(DETAIL);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if in_detail {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.detail_cursor = self.detail_cursor.saturating_sub(1);
                    self.detail_scroll.ensure_visible(self.detail_cursor);
                    return Outcome::Changed;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.detail_cursor =
                        (self.detail_cursor + 1).min(self.detail_rows.len().saturating_sub(1));
                    self.detail_scroll.ensure_visible(self.detail_cursor);
                    return Outcome::Changed;
                }
                KeyCode::PageDown => {
                    self.detail_scroll.page_down();
                    return Outcome::Changed;
                }
                KeyCode::PageUp => {
                    self.detail_scroll.page_up();
                    return Outcome::Changed;
                }
                KeyCode::Enter => {
                    if let (RowKey::Instance(id), Some(DetailRow::Pane { pane, .. })) = (
                        self.selected.clone(),
                        self.detail_rows.get(self.detail_cursor),
                    ) {
                        let pane = *pane;
                        if w.instance(&id).is_some_and(|i| i.status.is_live()) {
                            cx.go(Go::Attach {
                                instance: id,
                                pane: Some(pane),
                            });
                            return Outcome::Changed;
                        }
                    }
                    return self.activate(w, cx);
                }
                KeyCode::Esc | KeyCode::Left | KeyCode::BackTab => {
                    self.drawer_open = false;
                    cx.focus.focus(TREE);
                    return Outcome::Changed;
                }
                KeyCode::Tab => {
                    self.drawer_open = true;
                    cx.focus_next();
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if let Some(i) = action_focus {
            let (o, fired) = self.actions[i].on_key(key);
            if fired {
                let name = self.actions[i].id;
                let names = [
                    "reconnect",
                    "session",
                    "shell",
                    "inspect",
                    "stop",
                    "purge",
                    "launch",
                    "edit",
                    "prewarm",
                    "delete",
                    "new",
                ];
                for n in names {
                    if DETAIL.sub(n) == name {
                        return self.fire_action(n, w, cx);
                    }
                }
            }
            if o.consumed() {
                return o;
            }
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    if i > 0 {
                        cx.focus.focus(self.actions[i - 1].id);
                    } else {
                        cx.focus.focus(DETAIL);
                    }
                    return Outcome::Changed;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if i + 1 < self.actions.len() {
                        cx.focus.focus(self.actions[i + 1].id);
                    }
                    return Outcome::Changed;
                }
                KeyCode::Esc => {
                    self.drawer_open = false;
                    cx.focus.focus(TREE);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if focus == Some(ROSTER) {
            match key.code {
                KeyCode::Esc | KeyCode::Left => {
                    cx.focus.focus(TREE);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if key.is(KeyCode::Esc) {
            // top of the ladder: quit
            cx.go(Go::Quit);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        self.build_rows(w);
        for i in self.scroll.visible_range() {
            if Self::toggle_id(i) == id {
                if let RowKey::Workspace(ws) = self.rows[i].key.clone() {
                    self.selected = RowKey::Workspace(ws);
                    self.toggle_expand(ws);
                    self.build_rows(w);
                }
                cx.focus.focus(TREE);
                return Outcome::Changed;
            }
            if Self::row_id(i) == id {
                let same = self.selected == self.rows[i].key;
                self.selected = self.rows[i].key.clone();
                self.detail_cursor = 0;
                cx.focus.focus(TREE);
                self.drawer_open = false;
                if same {
                    return self.activate(w, cx);
                }
                return Outcome::Changed;
            }
        }
        if id == scrollbar::id_for(TREE) {
            let track = Rect::new(
                self.tree_area.right() - 1,
                self.tree_area.y,
                1,
                self.tree_area.height,
            );
            self.scroll
                .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
            return Outcome::Changed;
        }
        if id == scrollbar::id_for(DETAIL) {
            let track = Rect::new(
                self.detail_area.right() - 1,
                self.detail_area.y,
                1,
                self.detail_area.height,
            );
            self.detail_scroll.scroll_to(scrollbar::offset_for_click(
                track,
                pos,
                &self.detail_scroll,
            ));
            return Outcome::Changed;
        }
        if id == DETAIL {
            cx.focus.focus(DETAIL);
            self.drawer_open = true;
            return Outcome::Changed;
        }
        for i in 0..self.detail_rows.len() {
            if Self::detail_row_id(i) == id {
                let was = self.detail_cursor == i && cx.focus.is(DETAIL);
                self.detail_cursor = i;
                cx.focus.focus(DETAIL);
                if was
                    && let (RowKey::Instance(iid), Some(DetailRow::Pane { pane, .. })) =
                        (self.selected.clone(), self.detail_rows.get(i))
                {
                    let pane = *pane;
                    cx.go(Go::Attach {
                        instance: iid,
                        pane: Some(pane),
                    });
                }
                return Outcome::Changed;
            }
        }
        for i in 0..self.actions.len() {
            if self.actions[i].id == id {
                cx.focus.focus(id);
                if self.actions[i].on_click() {
                    let names = [
                        "reconnect",
                        "session",
                        "shell",
                        "inspect",
                        "stop",
                        "purge",
                        "launch",
                        "edit",
                        "prewarm",
                        "delete",
                        "new",
                    ];
                    for n in names {
                        if DETAIL.sub(n) == id {
                            return self.fire_action(n, w, cx);
                        }
                    }
                }
                return Outcome::Changed;
            }
        }
        if id == ROSTER {
            cx.focus.focus(ROSTER);
            return Outcome::Changed;
        }
        // roster rows: child ids by y; select the instance
        for i in w.running() {
            let _ = i;
        }
        if let Some(y) = (0..200usize).find(|k| ROSTER.child(*k) == id) {
            // find the running instance rendered at that y
            let running = w.running();
            let _ = y;
            if let Some(inst) = running.get(((pos.y as usize).saturating_sub(4)) / 4) {
                self.select_instance(&inst.id.clone(), w);
                cx.focus.focus(TREE);
                return Outcome::Changed;
            }
        }
        Outcome::Ignored
    }

    fn on_double_click(
        &mut self,
        id: WidgetId,
        _pos: Position,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        for i in self.scroll.visible_range() {
            if Self::row_id(i) == id {
                self.selected = self.rows[i].key.clone();
                return self.activate(w, cx);
            }
        }
        Outcome::Ignored
    }

    fn on_drag(&mut self, pressed: WidgetId, pos: Position, _w: &mut World) -> Outcome {
        if pressed == SEAM {
            let container = self.seam_container;
            return self.seam.on_drag(&mut self.split, container, 2, pos);
        }
        if pressed == scrollbar::id_for(TREE) {
            let track = Rect::new(
                self.tree_area.right() - 1,
                self.tree_area.y,
                1,
                self.tree_area.height,
            );
            self.scroll
                .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == TREE || id == scrollbar::id_for(TREE) {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        if id == DETAIL || id == scrollbar::id_for(DETAIL) {
            self.detail_scroll.scroll_by(delta as isize);
            return Outcome::Changed;
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
            ("launch", ModalResult::Picked(i)) => {
                let Some((agent, account)) = self.picker_targets.get(i).cloned() else {
                    return Outcome::Changed;
                };
                let Some((ws, role)) = self.pending_launch.take() else {
                    return Outcome::Changed;
                };
                let plan = match w.scenario {
                    crate::scenario::Scenario::LaunchFailure => LaunchPlan::FailNetwork,
                    crate::scenario::Scenario::HardCases => {
                        if w.op.session == crate::sim::onepassword::OpSession::Locked
                            && account.as_deref().is_some_and(|a| {
                                a.contains("work")
                                    || a.contains("grok")
                                    || a.contains("codex-primary")
                            })
                        {
                            LaunchPlan::CredentialsLocked
                        } else if role == "sre" {
                            LaunchPlan::BlockedSidecar
                        } else {
                            LaunchPlan::Clean
                        }
                    }
                    _ => LaunchPlan::Clean,
                };
                cx.go(Go::Launch {
                    workspace: ws,
                    role,
                    agent,
                    account,
                    plan,
                });
                Outcome::Changed
            }
            ("session", ModalResult::Picked(i)) => {
                let Some((agent, account)) = self.picker_targets.get(i).cloned() else {
                    return Outcome::Changed;
                };
                let Some(instance) = self.pending_session.take() else {
                    return Outcome::Changed;
                };
                let shell = account.as_deref() == Some("__shell__");
                cx.go(Go::NewSession {
                    instance,
                    agent: if shell { None } else { Some(agent) },
                    account: if shell { None } else { account },
                });
                Outcome::Changed
            }
            (
                "stop",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                let id = tag.key.clone();
                self.busy_rows
                    .push((RowKey::Instance(id.clone()), "stopping…"));
                w.schedule(1_800, Msg::Stopped { instance: id });
                cx.status("Stopping…");
                Outcome::Changed
            }
            (
                "purge",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                let id = tag.key.clone();
                self.busy_rows
                    .push((RowKey::Instance(id.clone()), "purging…"));
                w.schedule(2_200, Msg::Purged { instance: id });
                cx.status("Purging…");
                Outcome::Changed
            }
            (
                "delete",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                let id = tag.n as WorkspaceId;
                let name = w.workspace(id).map(|x| x.name.clone()).unwrap_or_default();
                w.workspaces.retain(|x| x.id != id);
                for i in w.instances.iter_mut() {
                    if i.workspace == Some(id) {
                        i.workspace = None;
                    }
                }
                self.selected = RowKey::CurrentDir;
                self.build_rows(w);
                cx.status(format!(
                    "Deleted workspace {name} · instances and files kept"
                ));
                Outcome::Changed
            }
            ("info", ModalResult::Info(InfoResult::Copy(v))) => {
                cx.copy(v);
                Outcome::Changed
            }
            (_, ModalResult::Cancelled)
            | (
                _,
                ModalResult::Dialog {
                    action: Some(0), ..
                },
            ) => {
                if tag.kind == "stop" || tag.kind == "purge" || tag.kind == "delete" {
                    cx.status("Cancelled · nothing changed");
                }
                self.pending_launch = None;
                self.pending_session = None;
                Outcome::Changed
            }
            _ => Outcome::Changed,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        self.build_rows(w);
        let t = ctx.theme;
        self.body_narrow = area.width < 100;
        self.wide = area.width >= 150;
        let focus = ctx.interaction.focus;
        if self.body_narrow {
            let drawer = self.drawer_visible(focus);
            let summary_h = 6u16.min(area.height / 3);
            let tree = Rect::new(
                area.x,
                area.y,
                area.width,
                area.height.saturating_sub(summary_h + 1),
            );
            self.draw_tree(tree, buf, ctx, w);
            let summary = Rect::new(area.x, tree.bottom() + 1, area.width, summary_h);
            if drawer {
                // the drawer covers the body; the tree is still a reachable stop
                self.draw_detail(
                    Rect::new(area.x, area.y, area.width, area.height),
                    buf,
                    ctx,
                    w,
                    true,
                );
            } else {
                self.draw_summary(summary, buf, ctx, w);
                // hidden-but-reachable detail stop
                ctx.control(DETAIL, Rect::ZERO, false);
            }
            return;
        }
        let seam_container = area;
        self.seam_container = seam_container;
        self.split.min_first = 28;
        self.split.min_second = 40;
        let (left, right) = self.split.horizontal(area, 2);
        let handle = self.split.handle(SplitDir::Horizontal, area, 2);
        self.draw_tree(left, buf, ctx, w);
        self.seam.render(
            Rect::new(handle.x + 1, handle.y, 1, handle.height),
            buf,
            ctx,
            t.canvas,
        );
        if self.wide {
            let (detail, roster) = Split::new(55, 40, 36).horizontal(right, 2);
            self.draw_detail(detail, buf, ctx, w, false);
            self.draw_roster(roster, buf, ctx, w);
        } else {
            self.draw_detail(right, buf, ctx, w, false);
        }
    }

    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint> {
        let mut v = vec![];
        if focus == Some(DETAIL) {
            v.push(hint("↑↓", "Move"));
            if matches!(self.selected, RowKey::Instance(_)) {
                v.push(hint("Enter", "Attach pane"));
            }
            v.push(hint("Tab", "Actions"));
            v.push(hint("Esc", "Back"));
            return v;
        }
        if self.actions.iter().any(|b| Some(b.id) == focus) {
            v.push(hint("← →", "Choose"));
            v.push(hint("Enter", "Run"));
            v.push(hint("Esc", "Back"));
            return v;
        }
        match &self.selected {
            RowKey::Instance(id) => {
                let live = w.instance(id).is_some_and(|i| i.status.is_live());
                let restorable = w
                    .instance(id)
                    .is_some_and(|i| i.status.reconnectable() && !i.status.is_live());
                if live {
                    v.push(hint("Enter", "Reconnect"));
                    v.push(hint("a", "New session"));
                    v.push(hint("x", "Shell"));
                    v.push(hint("t", "Stop"));
                } else if restorable {
                    v.push(hint("Enter", "Restore"));
                }
                v.push(hint("i", "Inspect"));
                v.push(hint("p", "Purge…"));
                v.push(hint("←", "Back"));
            }
            RowKey::Workspace(_) => {
                v.push(hint("Enter", "Launch"));
                v.push(hint("→", "Expand"));
                v.push(hint("e", "Edit"));
                v.push(hint("n", "New"));
                v.push(hint("d", "Delete…"));
                v.push(hint("w", "Prewarm"));
                v.push(hint("o", "GitHub"));
            }
            RowKey::CurrentDir => {
                v.push(hint("Enter", "Launch"));
                v.push(hint("n", "New"));
                if w.cwd_workspace().is_some() {
                    v.push(hint("e", "Edit"));
                }
            }
            RowKey::NewWorkspace => {
                v.push(hint("Enter", "Setup"));
            }
        }
        v.push(hint("Tab", "Details"));
        v.push(hint("c", "Accounts"));
        v.push(hint("u", "Usage"));
        v.push(hint("s", "Settings"));
        v.push(hint("?", "Help"));
        v.push(hint("q", "Quit"));
        v
    }

    fn crumb(&self, w: &World) -> String {
        match &self.selected {
            RowKey::CurrentDir => "Workspaces".into(),
            RowKey::Workspace(id) => format!(
                "Workspaces › {}",
                w.workspace(*id).map(|x| x.name.as_str()).unwrap_or("?")
            ),
            RowKey::Instance(id) => {
                let ws = w
                    .instance(id)
                    .and_then(|i| i.workspace)
                    .and_then(|x| w.workspace(x))
                    .map(|x| x.name.clone())
                    .unwrap_or("current directory".into());
                format!("Workspaces › {ws} › {}", id.trim_start_matches("jk-"))
            }
            RowKey::NewWorkspace => "Workspaces › new workspace".into(),
        }
    }

    fn strip_right(&self, w: &World) -> Vec<Segment> {
        let mut v = vec![];
        if !self.busy_rows.is_empty() {
            v.push(
                Segment::new(
                    format!(
                        "{} {}",
                        spinner_frame(w.clock.now_ms as u64 / 80),
                        self.busy_rows[0].1.trim_end_matches('…')
                    ),
                    Tone::Secondary,
                )
                .priority(6),
            );
        }
        v
    }

    fn animating(&self, _w: &World) -> bool {
        !self.busy_rows.is_empty() || self.still_inside.is_some()
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Quit);
        Outcome::Changed
    }
}

impl ManagerScreen {
    fn draw_summary(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.surface;
        let title = match &self.selected {
            RowKey::CurrentDir => "Current directory".to_owned(),
            RowKey::Workspace(id) => w.workspace(*id).map(|x| x.name.clone()).unwrap_or_default(),
            RowKey::Instance(id) => format!("Instance {}", id.trim_start_matches("jk-")),
            RowKey::NewWorkspace => "New workspace".into(),
        };
        let inner = Panel::card(Some(&title))
            .meta("Tab details")
            .render(area, buf, t);
        let lines: Vec<(String, Tone)> = match &self.selected {
            RowKey::CurrentDir | RowKey::Workspace(_) => match self.selected_workspace(w) {
                Some(ws) => vec![
                    (
                        format!(
                            "{} · {} · {}",
                            ws.workdir,
                            plural(ws.mounts.len(), "mount", "mounts"),
                            plural(ws.env_count(), "var", "vars")
                        ),
                        Tone::Secondary,
                    ),
                    (
                        format!(
                            "Roles {}{} · Auth {}",
                            ws.roles
                                .default
                                .as_deref()
                                .map(|d| format!("{d} ★"))
                                .unwrap_or("none".into()),
                            match &ws.roles.allowed {
                                crate::domain::workspace::AllowedRoles::All => " · all".to_owned(),
                                crate::domain::workspace::AllowedRoles::Custom(l) =>
                                    format!(" · {} allowed", l.len()),
                            },
                            w.account_for(Agent::ClaudeCode.provider(), Some(ws), None, None)
                                .label(&w.accounts)
                        ),
                        Tone::Muted,
                    ),
                ]
                .into_iter()
                .chain(w.instances_of(Some(ws.id)).iter().take(2).map(|i| {
                    (
                        format!(
                            "{} {}  {} · {} · {}",
                            if i.status.is_live() { "◉" } else { "◌" },
                            i.id.trim_start_matches("jk-"),
                            i.role,
                            i.agent.label(),
                            i.status.label()
                        ),
                        Tone::Secondary,
                    )
                }))
                .collect(),
                None => vec![(
                    format!(
                        "{} · not saved · Enter launches, n creates a workspace",
                        w.tilde(&w.cwd)
                    ),
                    Tone::Muted,
                )],
            },
            RowKey::Instance(id) => match w.instance(id) {
                Some(i) => vec![
                    (
                        format!(
                            "{} · {} · {} · {}",
                            i.role,
                            i.agent.label(),
                            i.status.label(),
                            i.dirty_summary()
                        ),
                        Tone::Secondary,
                    ),
                    (
                        match &i.daemon {
                            DaemonSnapshot::Tabs(tabs) => format!(
                                "daemon · {} · {}",
                                plural(tabs.len(), "tab", "tabs"),
                                plural(tabs.iter().map(|t| t.panes.len()).sum(), "pane", "panes")
                            ),
                            DaemonSnapshot::NoTabs => "daemon reports no tabs".into(),
                            DaemonSnapshot::Unavailable => "daemon unavailable".into(),
                        },
                        Tone::Muted,
                    ),
                ],
                None => vec![],
            },
            RowKey::NewWorkspace => vec![(
                "Enter starts the five-step create chain".into(),
                Tone::Muted,
            )],
        };
        for (i, (l, tone)) in lines.iter().enumerate() {
            if inner.y + i as u16 >= inner.bottom() {
                break;
            }
            buf.set_string(
                inner.x,
                inner.y + i as u16,
                truncate(l, inner.width as usize),
                Style::new().fg(t.tone(*tone)).bg(bg),
            );
        }
        ctx.clickable(DETAIL, area);
    }
}
