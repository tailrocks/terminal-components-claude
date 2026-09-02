//! Capsule: the attached in-Construct terminal workspace. Tabs, a nested
//! pane tree of simulated PTYs, the exact mode priority (dialog › drag ›
//! select › prefix › normal), scrollback, selection, copy, the command
//! palette and every Capsule dialog, detach, takeover and dirty exit.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{Theme, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::layout::SplitDir;
use junie_tui::ui::text::{truncate, truncate_middle, width};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::picker::{Picker, PickerItem};
use junie_tui::widgets::progress::{MeterTone, render_meter, spinner_frame};
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::segments::{self, Segment};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use super::modals::{ChoiceDialog, InfoDialog, InfoResult, modal_frame};
use super::{CustomModal, Cx, Go, Modal, ModalResult, ModalTag, Screen, plural};
use crate::domain::account::AccountId;
use crate::domain::agent::Agent;
use crate::domain::instance::AgentState;
use crate::domain::usage::{Freshness, QuotaStatus};
use crate::sim::pty::{
    Daemon, Direction, MAX_LABEL, MIN_PANE_COLS, MIN_PANE_ROWS, PaneId, PaneNode, Seam, nearest,
};
use crate::sim::world::{Msg, World};

pub const STRIP: WidgetId = WidgetId::of("capsule.strip");
pub const MENU: WidgetId = WidgetId::of("capsule.menu");
pub const CONTEXT: WidgetId = WidgetId::of("capsule.context");
pub const USAGE_CHIP: WidgetId = WidgetId::of("capsule.chip.usage");
pub const CONTAINER_CHIP: WidgetId = WidgetId::of("capsule.chip.container");
pub const PANES: WidgetId = WidgetId::of("capsule.panes");

const PREFIX_TIMEOUT_MS: i64 = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    PrefixAwait,
    Select,
    Drag,
    Dialog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Intent {
    NewTab,
    Split(SplitDir, bool),
}

pub struct CapsuleScreen {
    pub instance: String,
    pub prefix_until: Option<i64>,
    drag: Option<(Vec<u8>, Rect)>,
    selecting: Option<PaneId>,
    pub pane_rects: Vec<(PaneId, Rect)>,
    seams: Vec<Seam>,
    tab_areas: Vec<(usize, Rect)>,
    strip_first: usize,
    pub body: Rect,
    pending_intent: Option<Intent>,
    pending_agent: Option<Agent>,
    picker_accounts: Vec<AccountId>,
    palette_cmds: Vec<&'static str>,
    pub dialog_open: bool,
    pub takeover: Option<String>,
    export_kind: Option<(bool, bool)>,
    pub redraw_flash: u8,
    pending_spawn: Option<PendingSpawn>,
}

/// Intent, agent, account, workspace name and request time of a spawn that
/// waits for its picker chain.
type PendingSpawn = (Intent, Option<Agent>, Option<AccountId>, String, i64);

const PALETTE: [&str; 20] = [
    "New tab",
    "Split pane",
    "Zoom / unzoom pane",
    "Export file",
    "Export file and reveal",
    "Export file and open",
    "Export file under cursor",
    "Export file under cursor and reveal",
    "Export file under cursor and open",
    "Export selected file",
    "Export selected file and reveal",
    "Export selected file and open",
    "Stage image from clipboard path",
    "Paste image from host clipboard",
    "Stage image without pasting",
    "Open link under cursor",
    "Clear pane",
    "Usage",
    "Close",
    "Exit",
];

impl CapsuleScreen {
    pub fn new(instance: &str, w: &World, pane: Option<u64>) -> Self {
        let mut s = Self {
            instance: instance.to_owned(),
            prefix_until: None,
            drag: None,
            selecting: None,
            pane_rects: vec![],
            seams: vec![],
            tab_areas: vec![],
            strip_first: 0,
            body: Rect::ZERO,
            pending_intent: None,
            pending_agent: None,
            picker_accounts: vec![],
            palette_cmds: vec![],
            dialog_open: false,
            takeover: None,
            export_kind: None,
            redraw_flash: 0,
            pending_spawn: None,
        };
        let _ = w;
        if let Some(p) = pane {
            s.focus_pane_by_id(p);
        }
        s
    }

    fn focus_pane_by_id(&mut self, _p: u64) {}

    pub fn mode(&self) -> Mode {
        if self.dialog_open {
            Mode::Dialog
        } else if self.drag.is_some() {
            Mode::Drag
        } else if self.selecting.is_some() {
            Mode::Select
        } else if self.prefix_until.is_some() {
            Mode::PrefixAwait
        } else {
            Mode::Normal
        }
    }

    fn daemon<'a>(&self, w: &'a World) -> Option<&'a Daemon> {
        w.daemons.get(&self.instance)
    }

    fn daemon_mut<'a>(&self, w: &'a mut World) -> Option<&'a mut Daemon> {
        w.daemons.get_mut(&self.instance)
    }

    fn focused_pane(&self, w: &World) -> Option<PaneId> {
        self.daemon(w).and_then(|d| d.focused_pane())
    }

    fn account_suffix(w: &World, p: &crate::sim::pty::Pane) -> Option<String> {
        let id = p.proc.account.as_ref()?;
        let a = w.accounts.get(id)?;
        if w.accounts.by_provider(a.provider).count() >= 2 {
            Some(a.display_name.clone())
        } else {
            None
        }
    }

    fn layout(&mut self, body: Rect, w: &World) {
        self.pane_rects.clear();
        self.seams.clear();
        let Some(d) = self.daemon(w) else { return };
        let Some(tab) = d.active_tab() else { return };
        if let Some(z) = tab.zoomed {
            self.pane_rects.push((z, body));
            return;
        }
        let mut leaves = vec![];
        let mut seams = vec![];
        tab.root.layout(body, &mut leaves, &mut seams, &mut vec![]);
        self.pane_rects = leaves;
        self.seams = seams;
    }

    fn framed(&self, w: &World) -> bool {
        self.daemon(w)
            .and_then(|d| d.active_tab())
            .is_some_and(|t| t.leaves().len() > 1 || t.zoomed.is_some())
    }

    fn inner_of(&self, w: &World, r: Rect) -> Rect {
        if self.framed(w) {
            r.inner(ratatui::layout::Margin::new(1, 1))
        } else {
            r
        }
    }

    // -------------------------------------------------------- commands

    fn enter_prefix(&mut self, w: &World) {
        self.prefix_until = Some(w.now_ms() + PREFIX_TIMEOUT_MS);
    }

    fn spawn_flow(&mut self, intent: Intent, w: &World, cx: &mut Cx) {
        self.pending_intent = Some(intent);
        let title = match intent {
            Intent::NewTab => "New tab".to_owned(),
            Intent::Split(SplitDir::Horizontal, false) => "Split → Right".into(),
            Intent::Split(SplitDir::Horizontal, true) => "Split ← Left".into(),
            Intent::Split(SplitDir::Vertical, false) => "Split ↓ Below".into(),
            Intent::Split(SplitDir::Vertical, true) => "Split ↑ Above".into(),
        };
        let mut p = Picker::new(WidgetId::of("capsule.agent"), &title);
        p.width = 72;
        let ws = w
            .instance(&self.instance)
            .and_then(|i| i.workspace)
            .and_then(|x| w.workspace(x));
        let role = w.instance(&self.instance).map(|i| i.role.clone());
        let mut items = vec![];
        for a in Agent::ALL {
            let r = w.account_for(a.provider(), ws, role.as_deref(), None);
            let detail = match &r.account {
                Some(id) => w
                    .accounts
                    .get(id)
                    .map(|x| format!("{} · {}", x.display_name, r.level.label()))
                    .unwrap_or_default(),
                None => "needs account".into(),
            };
            items.push(PickerItem {
                label: a.label().into(),
                detail,
                glyph: "▪",
                group: "agents",
                tag: None,
                matched: vec![],
                disabled: false,
            });
        }
        items.push(PickerItem {
            label: "Shell".into(),
            detail: "zsh".into(),
            glyph: "$",
            group: "shells",
            tag: None,
            matched: vec![],
            disabled: false,
        });
        p.set_items(items);
        cx.open(Modal::Picker(p), ModalTag::new("agent"));
    }

    fn continue_spawn(&mut self, agent: Option<Agent>, w: &World, cx: &mut Cx) {
        match agent {
            None => self.spawn(None, None, w, cx),
            Some(a) => {
                let accounts: Vec<&crate::domain::account::Account> = w
                    .accounts
                    .by_provider(a.provider())
                    .filter(|x| x.origin == crate::domain::account::AccountOrigin::Registered)
                    .collect();
                if accounts.len() >= 2 {
                    self.pending_agent = Some(a);
                    let ws = w
                        .instance(&self.instance)
                        .and_then(|i| i.workspace)
                        .and_then(|x| w.workspace(x));
                    let role = w.instance(&self.instance).map(|i| i.role.clone());
                    let resolved = w.account_for(a.provider(), ws, role.as_deref(), None);
                    let mut p = Picker::new(
                        WidgetId::of("capsule.provider"),
                        &format!("Account for {}", a.label()),
                    );
                    p.searchable = false;
                    p.width = 72;
                    p.scope = Some("session choice › Role › Workspace › provider default".into());
                    let mut items = vec![];
                    self.picker_accounts.clear();
                    let mut cursor = 0;
                    for (i, acc) in accounts.iter().enumerate() {
                        let mut tags = vec![];
                        if acc.default_for_provider {
                            tags.push("provider default");
                        }
                        if resolved.account.as_deref() == Some(acc.id.as_str()) {
                            tags.push(resolved.level.label());
                            cursor = i;
                        }
                        items.push(PickerItem {
                            label: acc.display_name.clone(),
                            detail: format!(
                                "{} · {}{}",
                                acc.source.origin_label(),
                                acc.status_word(),
                                if tags.is_empty() {
                                    String::new()
                                } else {
                                    format!(" · {}", tags.join(" · "))
                                }
                            ),
                            glyph: if acc.default_for_provider { "★" } else { " " },
                            group: "",
                            tag: if acc.enabled { None } else { Some("disabled") },
                            matched: vec![],
                            disabled: !acc.enabled,
                        });
                        self.picker_accounts.push(acc.id.clone());
                    }
                    p.set_items(items);
                    p.cursor = cursor;
                    cx.open(Modal::Picker(p), ModalTag::new("provider"));
                } else {
                    let ws = w
                        .instance(&self.instance)
                        .and_then(|i| i.workspace)
                        .and_then(|x| w.workspace(x));
                    let role = w.instance(&self.instance).map(|i| i.role.clone());
                    let r = w.account_for(a.provider(), ws, role.as_deref(), None);
                    if r.account.is_none() && a.registerable() {
                        let d = Dialog::confirm(
                            WidgetId::of("capsule.spawnfail"),
                            &format!("{} could not start", a.label()),
                            &format!(
                                "No {} account is registered. Add one in Account & Usage Center, then try again.",
                                a.provider().short()
                            ),
                            "OK",
                        );
                        let mut d = d;
                        d.actions.remove(0);
                        d.cancel_index = Some(0);
                        d.initial_focus = d.actions[0].id;
                        cx.open(Modal::Dialog(d), ModalTag::new("spawnfail"));
                        return;
                    }
                    self.spawn(Some(a), r.account, w, cx);
                }
            }
        }
    }

    fn spawn(&mut self, agent: Option<Agent>, account: Option<AccountId>, w: &World, cx: &mut Cx) {
        let intent = self.pending_intent.take().unwrap_or(Intent::NewTab);
        let now = w.now_ms();
        let ws = self
            .daemon(w)
            .map(|d| d.workspace.clone())
            .unwrap_or_default();
        // split refusal by minimum pane size
        if let Intent::Split(dir, _) = intent
            && let Some(f) = self.focused_pane(w)
            && let Some((_, r)) = self.pane_rects.iter().find(|(id, _)| *id == f)
        {
            let inner = self.inner_of(w, *r);
            let (need_c, need_r, have_c, have_r) = match dir {
                SplitDir::Horizontal => (
                    MIN_PANE_COLS,
                    MIN_PANE_ROWS,
                    inner.width / 2 - 2,
                    inner.height.saturating_sub(2),
                ),
                SplitDir::Vertical => (
                    MIN_PANE_COLS,
                    MIN_PANE_ROWS,
                    inner.width.saturating_sub(2),
                    inner.height / 2 - 2,
                ),
            };
            if have_c < need_c {
                cx.error(format!(
                    "Cannot split: pane would be {have_c} columns, needs {need_c}"
                ));
                return;
            }
            if have_r < need_r {
                cx.error(format!(
                    "Cannot split: pane would be {have_r} rows, needs {need_r}"
                ));
                return;
            }
        }
        if let Some(t) = self.daemon(w).and_then(|d| d.active_tab())
            && let Intent::Split(..) = intent
            && t.zoomed.is_some()
            && t.zoomed != Some(t.focused)
        {
            cx.error("Cannot split: unzoom first");
            return;
        }
        let d = w.daemons.get(&self.instance);
        let _ = d;
        // mutate through a request: the app hands us &World here, so queue
        // the spawn as a message via status; simpler: perform it directly
        // in on_modal where we hold &mut World.
        self.pending_spawn = Some((intent, agent, account, ws, now));
        cx.status("spawning");
    }
}

impl CapsuleScreen {
    fn apply_pending_spawn(&mut self, w: &mut World, cx: &mut Cx) {
        let Some((intent, agent, account, _ws, now)) = self.pending_spawn.take() else {
            return;
        };
        let Some(d) = self.daemon_mut(w) else { return };
        match intent {
            Intent::NewTab => {
                d.new_tab(agent, account.clone(), now, false);
            }
            Intent::Split(dir, first) => {
                if d.split(dir, first, agent, account.clone(), now, false)
                    .is_none()
                {
                    cx.error("Cannot split: focused pane not found");
                }
            }
        }
        let label = agent
            .map(|a| a.label().to_owned())
            .unwrap_or("Shell".into());
        match account.and_then(|id| w.accounts.get(&id).map(|a| a.display_name.clone())) {
            Some(acc) => cx.status(format!("Started {label} · account {acc}")),
            None => cx.status(format!("Started {label}")),
        }
        crate::domain::fixtures::refresh_snapshots(w);
        let now_secs = w.clock.now_secs();
        if let Some(i) = w.instance_mut(&self.instance)
            && let Ok(s) = i.sessions.as_mut()
        {
            let id = format!("s-{:02}", s.len() + 6);
            s.push(crate::domain::instance::SessionRecord {
                id,
                agent,
                label: label.to_lowercase(),
                status: crate::domain::instance::SessionStatus::Active,
                started_secs: now_secs,
            });
        }
    }

    fn move_focus(&mut self, dir: Direction, w: &mut World, cx: &mut Cx) {
        let Some(from) = self.focused_pane(w) else {
            return;
        };
        if self
            .daemon(w)
            .and_then(|d| d.active_tab())
            .is_some_and(|t| t.zoomed.is_some())
        {
            cx.status("Unzoom to move focus");
            return;
        }
        if let Some(to) = nearest(&self.pane_rects, from, dir)
            && let Some(t) = self.daemon_mut(w).and_then(|d| d.active_tab_mut())
        {
            t.focused = to;
        }
    }

    fn resize(&mut self, dir: Direction, w: &mut World, cx: &mut Cx) {
        let Some(from) = self.focused_pane(w) else {
            return;
        };
        let Some(d) = self.daemon_mut(w) else { return };
        let Some(tab) = d.active_tab_mut() else {
            return;
        };
        if tab.zoomed.is_some() {
            cx.status("Unzoom to resize");
            return;
        }
        let mut path = vec![];
        if !tab.root.path_to(from, &mut path) {
            return;
        }
        // deepest split whose orientation matches and which the direction crosses
        let want_dir = match dir {
            Direction::Left | Direction::Right => SplitDir::Horizontal,
            Direction::Up | Direction::Down => SplitDir::Vertical,
        };
        let mut chosen: Option<(Vec<u8>, i16)> = None;
        for k in (0..path.len()).rev() {
            let node_path = &path[..k];
            let side = path[k];
            if let Some(PaneNode::Split { dir: sd, .. }) = tab.root.node_at_mut(node_path)
                && *sd == want_dir
            {
                let grows_first = matches!(dir, Direction::Right | Direction::Down);
                let delta = if (side == 0) == grows_first { 5 } else { -5 };
                chosen = Some((node_path.to_vec(), delta));
                break;
            }
        }
        let Some((p, delta)) = chosen else {
            cx.status("Nothing to resize in that direction");
            return;
        };
        let seam = self.seams.iter().find(|s| s.path == p).cloned();
        if let Some(PaneNode::Split { split, dir: sd, .. }) = tab.root.node_at_mut(&p) {
            let before = split.percent;
            split.percent = (split.percent as i16 + delta).clamp(5, 95) as u16;
            if let Some(s) = seam {
                let (a, b) = split.layout(*sd, s.container, 1);
                let too_small = match sd {
                    SplitDir::Horizontal => {
                        a.width < MIN_PANE_COLS + 2 || b.width < MIN_PANE_COLS + 2
                    }
                    SplitDir::Vertical => {
                        a.height < MIN_PANE_ROWS + 2 || b.height < MIN_PANE_ROWS + 2
                    }
                };
                if too_small {
                    split.percent = before;
                    cx.error("Cannot resize: pane at minimum size");
                }
            }
        }
    }

    fn toggle_zoom(&mut self, w: &mut World, cx: &mut Cx) {
        let Some(t) = self.daemon_mut(w).and_then(|d| d.active_tab_mut()) else {
            return;
        };
        if t.zoomed.is_some() {
            t.zoomed = None;
            cx.status("Unzoomed");
        } else if t.leaves().len() > 1 {
            t.zoomed = Some(t.focused);
            cx.status("Zoomed · z restores the layout");
        } else {
            cx.status("Nothing to zoom: one pane");
        }
    }

    fn open_palette(&mut self, w: &World, cx: &mut Cx) {
        let mut p = Picker::new(WidgetId::of("capsule.palette"), "Command palette");
        p.width = 60;
        p.placeholder = "Type to filter commands…".into();
        p.empty_text = "No matching commands".into();
        self.palette_cmds = PALETTE.to_vec();
        p.set_items(self.palette_items("", w));
        p.scope = Some(format!("{} of 20", PALETTE.len()));
        cx.open(Modal::Picker(p), ModalTag::new("palette"));
    }

    fn palette_items(&mut self, query: &str, w: &World) -> Vec<PickerItem> {
        let single = self
            .daemon(w)
            .and_then(|d| d.active_tab())
            .is_some_and(|t| t.leaves().len() == 1);
        let has_sel = self
            .focused_pane(w)
            .and_then(|p| self.daemon(w).and_then(|d| d.pane(p)))
            .is_some_and(|p| p.term.has_selection());
        let q = query.to_lowercase();
        let mut out = vec![];
        self.palette_cmds.clear();
        for c in PALETTE {
            let label = if c == "Close" && single {
                "Close tab"
            } else {
                c
            };
            if !q.is_empty() && !label.to_lowercase().contains(&q) {
                continue;
            }
            let (detail, disabled) = match c {
                "New tab" => ("Ctrl+B c", false),
                "Split pane" => ("Ctrl+B \" %", false),
                "Zoom / unzoom pane" => ("Ctrl+B z", false),
                "Export file under cursor"
                | "Export file under cursor and reveal"
                | "Export file under cursor and open" => ("no file under cursor", true),
                "Export selected file"
                | "Export selected file and reveal"
                | "Export selected file and open" => {
                    if has_sel {
                        ("selection", false)
                    } else {
                        ("no selection", true)
                    }
                }
                "Open link under cursor" => ("no link under cursor", true),
                "Clear pane" => ("Ctrl+B Ctrl+L", false),
                "Usage" => ("Ctrl+B u", false),
                "Close" => ("Ctrl+B x", false),
                "Exit" => ("Ctrl+Q", false),
                _ => ("host", false),
            };
            let matched: Vec<usize> = if q.is_empty() {
                vec![]
            } else {
                label
                    .to_lowercase()
                    .find(&q)
                    .map(|p| (p..p + q.len()).collect())
                    .unwrap_or_default()
            };
            out.push(PickerItem {
                label: label.into(),
                detail: detail.into(),
                glyph: " ",
                group: "",
                tag: None,
                matched,
                disabled,
            });
            self.palette_cmds.push(c);
        }
        out
    }

    fn run_palette(&mut self, cmd: &str, w: &mut World, cx: &mut Cx) {
        match cmd {
            "New tab" => self.spawn_flow(Intent::NewTab, w, cx),
            "Split pane" => {
                let mut p = Picker::new(WidgetId::of("capsule.split"), "Split pane");
                p.searchable = false;
                p.width = 40;
                p.set_items(
                    ["→ Right", "← Left", "↓ Below", "↑ Above"]
                        .iter()
                        .map(|l| PickerItem {
                            label: (*l).into(),
                            detail: String::new(),
                            glyph: " ",
                            group: "",
                            tag: None,
                            matched: vec![],
                            disabled: false,
                        })
                        .collect(),
                );
                cx.open(Modal::Picker(p), ModalTag::new("split"));
            }
            "Zoom / unzoom pane" => self.toggle_zoom(w, cx),
            "Export file" | "Export file and reveal" | "Export file and open" => {
                let reveal = cmd.ends_with("reveal");
                let open = cmd.ends_with("open");
                self.export_kind = Some((reveal, open));
                let input = TextInput::new(WidgetId::of("capsule.export.path"), "Path")
                    .placeholder("workspace path or /jackin/run/…")
                    .help("Copies to ~/Downloads/jackin/ on the host")
                    .plain_label();
                let d = Dialog::prompt(WidgetId::of("capsule.export"), cmd, input, "Export");
                cx.open(Modal::Dialog(d), ModalTag::new("export"));
            }
            "Export selected file"
            | "Export selected file and reveal"
            | "Export selected file and open" => {
                let text = self
                    .focused_pane(w)
                    .and_then(|p| self.daemon(w).and_then(|d| d.pane(p)))
                    .and_then(|p| p.term.selected_text())
                    .unwrap_or_default();
                let path = text.lines().next().unwrap_or("").trim().to_owned();
                self.export_kind = Some((cmd.ends_with("reveal"), cmd.ends_with("open")));
                self.finish_export(&path, cx);
            }
            "Stage image from clipboard path"
            | "Paste image from host clipboard"
            | "Stage image without pasting" => {
                cx.status(format!(
                    "{cmd}: host clipboard has no image · nothing staged"
                ));
            }
            "Clear pane" => {
                if let Some(p) = self.focused_pane(w)
                    && let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(p))
                {
                    pane.clear();
                }
            }
            "Usage" => self.open_usage(w, cx),
            "Close" | "Close tab" => {
                let single = self
                    .daemon(w)
                    .and_then(|d| d.active_tab())
                    .is_some_and(|t| t.leaves().len() == 1);
                if single {
                    self.confirm_close(true, w, cx);
                } else {
                    let mut p = Picker::new(WidgetId::of("capsule.closetarget"), "Close");
                    p.searchable = false;
                    p.width = 40;
                    p.set_items(
                        ["Close pane", "Close tab"]
                            .iter()
                            .map(|l| PickerItem {
                                label: (*l).into(),
                                detail: String::new(),
                                glyph: " ",
                                group: "",
                                tag: None,
                                matched: vec![],
                                disabled: false,
                            })
                            .collect(),
                    );
                    cx.open(Modal::Picker(p), ModalTag::new("closetarget"));
                }
            }
            "Exit" => self.request_exit(w, cx),
            _ => cx.status(format!("{cmd}: not available in the preview")),
        }
    }

    fn finish_export(&mut self, path: &str, cx: &mut Cx) {
        let (reveal, open) = self.export_kind.take().unwrap_or((false, false));
        if path.is_empty() {
            cx.error("Enter a path");
            return;
        }
        if path.contains("..") || path.starts_with("/etc") {
            cx.error("Export failed: path is outside the workspace");
            return;
        }
        if path.ends_with('/') {
            cx.error("Export failed: is a directory");
            return;
        }
        if path == "cancel" {
            cx.status("Export cancelled");
            return;
        }
        let mut s = format!(
            "Exported to ~/Downloads/jackin/payments-platform/{}",
            path.trim_start_matches('/')
        );
        if reveal {
            s.push_str(" · revealed in Finder");
        }
        if open {
            s.push_str(" · opened on host");
        }
        cx.status(s);
    }

    fn confirm_close(&mut self, tab: bool, w: &World, cx: &mut Cx) {
        let d = if tab {
            Dialog::destructive(
                WidgetId::of("capsule.closetab"),
                "Close tab?",
                "Reap every pane in this tab. Unsaved state across all panes is lost.",
                "Close tab",
            )
        } else {
            Dialog::destructive(
                WidgetId::of("capsule.closepane"),
                "Close pane?",
                "Reap the focused pane's agent. Unsaved state in that pane is lost.",
                "Close pane",
            )
        };
        let _ = w;
        cx.open(
            Modal::Dialog(d),
            ModalTag::new(if tab { "closetab" } else { "closepane" }),
        );
    }

    fn is_dirty(&self, w: &World) -> bool {
        w.instance(&self.instance).is_some_and(|i| i.is_dirty())
            || self
                .daemon(w)
                .is_some_and(|d| !d.touched_files().is_empty())
    }

    fn request_exit(&mut self, w: &World, cx: &mut Cx) {
        if self.is_dirty(w) {
            let inst = w.instance(&self.instance);
            let touched = self
                .daemon(w)
                .map(|d| d.touched_files())
                .unwrap_or_default();
            let changed = inst.map(|i| i.uncommitted).unwrap_or(0) + touched.len();
            let unpushed = inst.map(|i| i.unpushed).unwrap_or(0);
            let ws = self
                .daemon(w)
                .map(|d| d.workspace.clone())
                .unwrap_or_default();
            let c = ChoiceDialog::new(
                WidgetId::of("capsule.exitdirty"),
                "Unsaved work — exit?",
                "",
                &[
                    "Start a new agent",
                    "Inspect changes",
                    "Exit & keep changes",
                    "Exit & discard changes",
                ],
                0,
            )
            .line(
                format!("{ws}   • {changed} changed · {unpushed} unpushed"),
                Tone::Warning,
            )
            .buttons(
                vec![
                    Button::subtle(WidgetId::of("capsule.exitdirty").sub("cancel"), "Cancel"),
                    Button::primary(WidgetId::of("capsule.exitdirty").sub("ok"), "Choose"),
                ],
                0,
            )
            .option_tones(vec![Tone::Normal, Tone::Normal, Tone::Normal, Tone::Error]);
            cx.open(Modal::Choice(c), ModalTag::new("exitdirty"));
        } else {
            let mut d = Dialog::destructive(
                WidgetId::of("capsule.exit"),
                "Exit jackin❯?",
                "! Exiting force-stops the container immediately.\n! Work not saved outside the container will be lost.",
                "Exit",
            );
            d.width = 60;
            cx.open(Modal::Dialog(d), ModalTag::new("exit"));
        }
    }

    fn open_usage(&mut self, w: &World, cx: &mut Cx) {
        let mut accounts: Vec<AccountId> = vec![];
        if let Some(d) = self.daemon(w) {
            for p in &d.panes {
                if let Some(a) = &p.proc.account
                    && !accounts.contains(a)
                {
                    accounts.push(a.clone());
                }
            }
        }
        if accounts.is_empty() {
            for a in w.accounts.sorted() {
                if a.default_for_provider {
                    accounts.push(a.id.clone());
                }
            }
        }
        let focused = self
            .focused_pane(w)
            .and_then(|p| self.daemon(w).and_then(|d| d.pane(p)))
            .and_then(|p| p.proc.account.clone());
        let mut dlg = UsageDialog::new(accounts.clone(), w);
        if let Some(f) = focused
            && let Some(i) = accounts.iter().position(|a| *a == f)
        {
            dlg.tab = i + 1;
        }
        cx.open(Modal::Custom(Box::new(dlg)), ModalTag::new("usage"));
    }

    fn open_container_info(&mut self, w: &World, cx: &mut Cx) {
        let Some(i) = w.instance(&self.instance) else {
            return;
        };
        let ws = i.workspace.and_then(|x| w.workspace(x));
        let focused = self
            .focused_pane(w)
            .and_then(|p| self.daemon(w).and_then(|d| d.pane(p)));
        let agent = match focused {
            Some(p) => match p.proc.agent {
                Some(a) => format!(
                    "{} ({})",
                    a.short(),
                    p.proc
                        .account
                        .as_ref()
                        .and_then(|id| w.accounts.get(id))
                        .map(|x| x.title())
                        .unwrap_or(a.label().into())
                ),
                None => "(shell)".into(),
            },
            None => "(none)".into(),
        };
        let props = vec![
            Prop::new("Container", i.container_id()).copyable(),
            Prop::new(
                "Container ID",
                format!("3f9c{}e21a", &i.run_id.replace('-', "")[..8]),
            )
            .copyable(),
            Prop::new("Role", i.role.clone()),
            Prop::new("Agent", agent),
            Prop::new("Workdir", i.workdir.clone()),
            Prop::new("Instance", i.id.trim_start_matches("jk-").to_owned()),
            Prop::new("Capsule", "0.9.2"),
            Prop::new("Invocation ID", format!("{}-4f11", i.run_id)).copyable(),
            Prop::new(
                "Host log",
                format!("file:///Users/alexey/.jackin/logs/{}.log", i.run_id),
            ),
        ];
        let _ = ws;
        let d = InfoDialog::new(WidgetId::of("capsule.info"), "Debug info", props)
            .meta("Enter copies the row · y copies");
        cx.open(Modal::Info(d), ModalTag::new("info"));
    }

    fn open_github(&mut self, w: &World, cx: &mut Cx) {
        let Some(i) = w.instance(&self.instance) else {
            return;
        };
        let repo = w.github.iter().find(|r| {
            r.full_name
                .ends_with(&i.workdir.trim_start_matches("/workspace/").to_owned())
        });
        let branch = i.branch.clone().unwrap_or(i.default_branch.clone());
        let mut props = vec![Prop::new("Branch", branch.clone())];
        match &i.pr {
            Some((n, title)) => {
                props.push(Prop::new("Pull Request", format!("#{n}")));
                props.push(Prop::new("PR Title", title.clone()));
                if let Some(r) = repo {
                    props.push(Prop::new("GitHub URL", format!("{}/pull/{n}", r.url)).copyable());
                }
                props.push(Prop::new("CI Status", "✓ passing").tone(Tone::Success));
            }
            None => {
                props.push(Prop::new("Pull Request", "(none)").tone(Tone::Muted));
                if let Some(r) = repo {
                    props.push(
                        Prop::new("GitHub URL", format!("{}/tree/{branch}", r.url)).copyable(),
                    );
                }
                props.push(Prop::new("CI Status", "(unknown)").tone(Tone::Muted));
            }
        }
        let d = InfoDialog::new(WidgetId::of("capsule.github"), "GitHub context", props)
            .action(Button::secondary(
                WidgetId::of("capsule.github").sub("open"),
                "Open PR…",
            ))
            .meta("open uses a gated host action");
        cx.open(Modal::Info(d), ModalTag::new("github"));
    }

    fn detach(&mut self, cx: &mut Cx) {
        cx.go(Go::Detach);
    }

    fn end_instance(&mut self, purge: bool, cx: &mut Cx) {
        cx.go(Go::InstanceEnded {
            instance: self.instance.clone(),
            purge,
        });
    }

    fn handle_prefix_cmd(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        self.prefix_until = None;
        if key.ctrl_char('b') {
            // literal prefix forwarded to the pane
            return Outcome::Changed;
        }
        if key.ctrl_char('l') {
            if let Some(p) = self.focused_pane(w)
                && let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(p))
            {
                pane.clear();
            }
            return Outcome::Changed;
        }
        match key.code {
            KeyCode::Char('c') => self.spawn_flow(Intent::NewTab, w, cx),
            KeyCode::Char('n') => self.next_tab(w, 1),
            KeyCode::Char('p') => self.next_tab(w, -1),
            KeyCode::Char('x') => {
                let single = self
                    .daemon(w)
                    .and_then(|d| d.active_tab())
                    .is_some_and(|t| t.leaves().len() == 1);
                self.confirm_close(single, w, cx);
            }
            KeyCode::Char('&') => self.confirm_close(true, w, cx),
            KeyCode::Char('"') => self.spawn_flow(Intent::Split(SplitDir::Vertical, false), w, cx),
            KeyCode::Char('%') => {
                self.spawn_flow(Intent::Split(SplitDir::Horizontal, false), w, cx)
            }
            KeyCode::Char('z') => self.toggle_zoom(w, cx),
            KeyCode::Char('h') => self.move_focus(Direction::Left, w, cx),
            KeyCode::Char('j') => self.move_focus(Direction::Down, w, cx),
            KeyCode::Char('k') => self.move_focus(Direction::Up, w, cx),
            KeyCode::Char('l') => self.move_focus(Direction::Right, w, cx),
            KeyCode::Char('d') => self.detach(cx),
            KeyCode::Char('u') => self.open_usage(w, cx),
            KeyCode::Char(' ') | KeyCode::Char(':') => self.open_palette(w, cx),
            KeyCode::Char('r') => {
                self.redraw_flash = 3;
                cx.status("Redrawn");
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let n = if c == '0' {
                    10
                } else {
                    c as usize - '0' as usize
                };
                let count = self.daemon(w).map(|d| d.tabs.len()).unwrap_or(0);
                if n >= 1 && n <= count {
                    if let Some(d) = self.daemon_mut(w) {
                        d.active = n - 1;
                    }
                } else {
                    cx.status(format!("No tab {n}"));
                }
            }
            KeyCode::Esc => {}
            KeyCode::Char(c) => cx.status(format!("Not a prefix command: {c}")),
            _ => {}
        }
        Outcome::Changed
    }

    fn next_tab(&mut self, w: &mut World, delta: isize) {
        if let Some(d) = self.daemon_mut(w)
            && !d.tabs.is_empty()
        {
            let n = d.tabs.len() as isize;
            d.active = ((d.active as isize + delta + n) % n) as usize;
        }
    }

    fn forward_key(&mut self, key: &Key, w: &mut World) -> Outcome {
        let Some(p) = self.focused_pane(w) else {
            return Outcome::Consumed;
        };
        let now = w.now_ms();
        let ws = self
            .daemon(w)
            .map(|d| d.workspace.clone())
            .unwrap_or_default();
        self.prime(p, w);
        let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(p)) else {
            return Outcome::Consumed;
        };
        // scrollback keys are intercepted while scrolled back
        if !pane.term.follow {
            match key.code {
                KeyCode::Up => {
                    pane.term.on_wheel(-1);
                    return Outcome::Changed;
                }
                KeyCode::Down => {
                    pane.term.on_wheel(1);
                    return Outcome::Changed;
                }
                KeyCode::PageUp => {
                    pane.term.scroll.page_up();
                    return Outcome::Changed;
                }
                KeyCode::PageDown => {
                    pane.term.scroll.page_down();
                    pane.term.follow = pane.term.is_at_tail();
                    return Outcome::Changed;
                }
                KeyCode::Home => {
                    pane.term.scroll.jump_start();
                    return Outcome::Changed;
                }
                KeyCode::End | KeyCode::Esc => {
                    pane.term.set_follow(true);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if pane.term.has_selection() {
            if key.is(KeyCode::Esc) {
                pane.term.clear_selection();
                return Outcome::Changed;
            }
            if key.is_char('y')
                && key.plain()
                && let Some(t) = pane.term.selected_text()
            {
                w.clipboard = Some(t);
                return Outcome::Changed;
            }
        }
        match key.code {
            KeyCode::Char(c) if !key.ctrl() && !key.alt() => {
                pane.term.clear_selection();
                pane.type_char(c, now, &ws);
            }
            KeyCode::Backspace => pane.backspace(),
            KeyCode::Enter => pane.commit(now, &ws),
            KeyCode::Esc => {
                pane.term.set_follow(true);
            }
            _ => return Outcome::Consumed,
        }
        Outcome::Changed
    }

    /// The pane's viewport in the world never renders (a copy does), so give
    /// it the geometry of the last frame before it handles an event.
    fn prime(&self, pid: PaneId, w: &mut World) {
        let Some((_, r)) = self.pane_rects.iter().find(|(p, _)| *p == pid).copied() else {
            return;
        };
        let inner = self.inner_of(w, r);
        if let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(pid)) {
            pane.term.set_area(inner);
        }
    }

    fn pane_at(&self, pos: Position) -> Option<PaneId> {
        self.pane_rects
            .iter()
            .find(|(_, r)| r.contains(pos))
            .map(|(id, _)| *id)
    }

    // ---------------------------------------------------------- render

    fn draw_strip(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        fill(buf, area, t.base());
        let Some(d) = self.daemon(w) else { return };
        self.tab_areas.clear();
        let y = area.y;
        // baseline
        for x in area.left()..area.right() {
            buf.set_string(x, y + 1, "─", Style::new().fg(t.border_subtle).bg(bg));
        }
        let mut x = area.x + 1;
        buf.set_string(x, y, "▪", t.accent_fg().bg(bg));
        buf.set_string(x + 2, y, "jackin❯", t.title().bg(bg));
        x += 12;
        // menu button on the right
        // `≡` is single-width everywhere; `☰` is East-Asian-ambiguous and
        // ratatui would reserve a hidden second cell for it
        let menu = if self.mode() == Mode::PrefixAwait {
            " prefix… "
        } else {
            "  ≡ Menu "
        };
        let mw = width(menu) as u16;
        let menu_x = area.right().saturating_sub(mw + 1);
        let hovered = ctx.interaction.hovered(MENU);
        let ms = if self.mode() == Mode::PrefixAwait {
            t.title().bg(bg)
        } else if hovered {
            t.primary().bg(t.surface_elevated)
        } else {
            t.secondary().bg(bg)
        };
        buf.set_string(menu_x, y, menu, ms);
        ctx.clickable(MENU, Rect::new(menu_x, y, mw, 1));
        // tab cells
        let labels: Vec<(String, AgentState)> = d
            .tabs
            .iter()
            .map(|tab| {
                (
                    d.tab_label(tab, &|p| Self::account_suffix(w, p)),
                    d.tab_state(tab),
                )
            })
            .collect();
        let cell_w = |i: usize, l: &str| -> u16 {
            let idx = if i <= 9 { 2 } else { 0 };
            (idx + width(l) + 4) as u16
        };
        let avail_right = menu_x.saturating_sub(5);
        // overflow window
        if d.active < self.strip_first {
            self.strip_first = d.active;
        }
        loop {
            let mut xx = x + if self.strip_first > 0 { 4 } else { 0 };
            let mut fit = 0;
            for (i, (l, _)) in labels.iter().enumerate().skip(self.strip_first) {
                let cw = cell_w(i, l);
                if xx + cw > avail_right {
                    break;
                }
                xx += cw + 1;
                fit += 1;
            }
            if fit == 0 && !labels.is_empty() {
                fit = 1;
            }
            if d.active >= self.strip_first + fit && self.strip_first + 1 < labels.len() {
                self.strip_first += 1;
                continue;
            }
            let hidden_left = self.strip_first;
            let hidden_right = labels.len().saturating_sub(self.strip_first + fit);
            if hidden_left > 0 {
                let s = format!("‹{hidden_left:<2}");
                buf.set_string(x, y, &s, t.muted().bg(bg));
                ctx.clickable(STRIP.sub("left"), Rect::new(x, y, 3, 1));
                x += 4;
            }
            for (i, (l, st)) in labels.iter().enumerate().skip(self.strip_first).take(fit) {
                let cw = cell_w(i, l);
                let r = Rect::new(x, y, cw, 1);
                let tid = STRIP.child(i);
                let active = i == d.active;
                let hov = ctx.interaction.hovered(tid);
                let mut style = Style::new().bg(bg).fg(if active || hov {
                    t.text_primary
                } else {
                    t.text_secondary
                });
                if hov && !active {
                    style = style.bg(t.lift(bg));
                }
                if active {
                    style = style.add_modifier(Modifier::BOLD);
                }
                fill(buf, r, style);
                let mut cx_ = x + 1;
                if i < 10 {
                    let idx = if i == 9 {
                        "0".to_owned()
                    } else {
                        (i + 1).to_string()
                    };
                    buf.set_string(
                        cx_,
                        y,
                        &idx,
                        style.fg(t.text_muted).remove_modifier(Modifier::BOLD),
                    );
                    cx_ += 2;
                }
                buf.set_string(cx_, y, l, style);
                cx_ += width(l) as u16 + 1;
                let (g, gt) = match st {
                    AgentState::Blocked => ("●", t.warning),
                    AgentState::Done => ("○", t.text_secondary),
                    AgentState::Working => ("▶", t.accent),
                    AgentState::Idle => ("◆", t.text_muted),
                    AgentState::Unknown => (" ", t.text_muted),
                };
                buf.set_string(cx_, y, g, style.fg(gt).remove_modifier(Modifier::BOLD));
                if active {
                    for xx in x + 1..x + cw - 1 {
                        buf.set_string(xx, y + 1, "━", Style::new().fg(t.accent).bg(bg));
                    }
                }
                ctx.clickable(tid, r);
                self.tab_areas.push((i, r));
                x += cw + 1;
            }
            if hidden_right > 0 {
                let s = format!("{hidden_right:>2}›");
                buf.set_string(menu_x.saturating_sub(4), y, &s, t.muted().bg(bg));
                ctx.clickable(
                    STRIP.sub("right"),
                    Rect::new(menu_x.saturating_sub(4), y, 3, 1),
                );
            }
            break;
        }
    }

    fn draw_panes(&mut self, body: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        fill(buf, body, t.base());
        self.layout(body, w);
        let Some(d) = self.daemon(w) else { return };
        if d.tabs.is_empty() {
            let e =
                EmptyState::new("No sessions").hint("Ctrl+B c starts an agent · Ctrl+B d detaches");
            empty::render(body, buf, t, &e, t.canvas);
            return;
        }
        let framed = self.framed(w);
        let Some(tab) = d.active_tab() else { return };
        let rects = self.pane_rects.clone();
        let dialog = self.mode() == Mode::Dialog;
        for (pid, r) in &rects {
            let Some(pane) = d.pane(*pid) else { continue };
            let focused = tab.focused == *pid;
            let inner = if framed {
                r.inner(ratatui::layout::Margin::new(1, 1))
            } else {
                *r
            };
            if framed {
                let hovered = ctx.interaction.hovered(PANES.child(*pid as usize));
                let block = ratatui::widgets::Block::new()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(t.border(focused).bg(t.canvas));
                ratatui::widgets::Widget::render(block, *r, buf);
                let (g, gt) = match pane.state() {
                    AgentState::Blocked => ("●", t.warning),
                    AgentState::Done => ("○", t.text_secondary),
                    AgentState::Working => ("▶", t.accent),
                    AgentState::Idle => ("◆", t.text_muted),
                    AgentState::Unknown => ("", t.text_muted),
                };
                let mut title = format!(" {}", pane.label());
                if let Some(s) = Self::account_suffix(w, pane) {
                    title.push_str(&format!(" ({s})"));
                }
                title.push(' ');
                let ts = if focused {
                    t.title().bg(t.canvas)
                } else if hovered {
                    t.primary().bg(t.canvas)
                } else {
                    t.secondary().bg(t.canvas)
                };
                if r.width > 6 {
                    buf.set_string(
                        r.x + 2,
                        r.y,
                        truncate(&title, r.width.saturating_sub(6) as usize),
                        ts,
                    );
                    if !g.is_empty() {
                        buf.set_string(
                            r.x + 2 + width(&title) as u16,
                            r.y,
                            g,
                            Style::new().fg(gt).bg(t.canvas),
                        );
                    }
                }
                let mut meta = String::new();
                if !pane.term.follow {
                    let range = pane.term.scroll.visible_range();
                    meta = format!(
                        "{}–{} of {}",
                        junie_tui::ui::text::thousands(range.start + 1),
                        junie_tui::ui::text::thousands(range.end),
                        junie_tui::ui::text::thousands(pane.term.scroll.content_len)
                    );
                }
                if tab.zoomed.is_some() {
                    meta = if meta.is_empty() {
                        "zoomed".into()
                    } else {
                        format!("{meta} · zoomed")
                    };
                }
                if !meta.is_empty() {
                    let m = format!(" {meta} ");
                    let mw = width(&m) as u16;
                    if r.width > mw + width(&title) as u16 + 6 {
                        buf.set_string(
                            r.right().saturating_sub(mw + 2),
                            r.y,
                            &m,
                            t.faint().bg(t.canvas),
                        );
                    }
                }
            }
            // the terminal body
            let mut term = pane.term.clone();
            term.caret_visible = pane.term.caret_visible
                && !dialog
                && focused
                && self.selecting.is_none()
                && self.drag.is_none();
            let saved_focus = ctx.interaction.focus;
            // the viewport is not a ring stop: the pane is
            ctx.inert = true;
            term.render(inner, buf, ctx, t.canvas);
            ctx.inert = false;
            if focused
                && term.follow
                && term.caret_visible
                && let Some(c) = term.caret
            {
                // re-place the hardware cursor (render skipped it while inert)
                let li = term.scroll.visible_range();
                let line_row = c.line.saturating_sub(li.start);
                if li.contains(&c.line) {
                    let cx_ = inner.x + (c.col as u16).min(inner.width.saturating_sub(1));
                    ctx.set_cursor(Position::new(cx_, inner.y + line_row as u16));
                }
            }
            let _ = saved_focus;
            // scrollbar while scrolled back
            if !pane.term.follow {
                let sb = Rect::new(inner.right().saturating_sub(1), inner.y, 1, inner.height);
                scrollbar::render_vertical(
                    sb,
                    buf,
                    ctx,
                    PANES.child(*pid as usize),
                    &pane.term.scroll,
                    focused,
                );
            }
            ctx.clickable(PANES.child(*pid as usize), *r);
            ctx.scrollable(PANES.child(*pid as usize), *r);
        }
        // seams on top
        let seams = self.seams.clone();
        for (i, s) in seams.iter().enumerate() {
            let sid = PANES.sub("seam").child(i);
            let hovered = ctx.interaction.hovered(sid)
                || self.drag.as_ref().is_some_and(|(p, _)| *p == s.path);
            let glyph = match (s.dir, hovered) {
                (SplitDir::Horizontal, false) => "│",
                (SplitDir::Horizontal, true) => "┃",
                (SplitDir::Vertical, false) => "─",
                (SplitDir::Vertical, true) => "━",
            };
            for pos in s.handle.positions() {
                buf.set_string(pos.x, pos.y, glyph, t.border(hovered).bg(t.canvas));
            }
            ctx.clickable(sid, s.handle);
        }
        ctx.control(PANES, Rect::new(body.x, body.y, 1, 1), false);
    }

    fn draw_context(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let Some(i) = w.instance(&self.instance) else {
            return;
        };
        let mut left = vec![];
        let branch = i.branch.clone().unwrap_or(i.default_branch.clone());
        if branch != i.default_branch {
            let text = match &i.pr {
                Some((n, title)) => format!("PR #{n} · {title}"),
                None => format!("Branch · {branch}"),
            };
            left.push(
                Segment::new(text, Tone::Secondary)
                    .clickable(CONTEXT)
                    .priority(9),
            );
        }
        let mut right = vec![];
        // usage chip for the focused pane's account
        let acc = self
            .focused_pane(w)
            .and_then(|p| self.daemon(w).and_then(|d| d.pane(p)))
            .and_then(|p| p.proc.account.clone())
            .and_then(|id| w.accounts.get(&id).cloned())
            .or_else(|| {
                w.accounts
                    .default_for(crate::domain::agent::Provider::Anthropic)
                    .cloned()
            });
        if let Some(a) = acc {
            let mut parts = vec![];
            for win in a.usage.windows.iter().take(2) {
                if let Some(p) = win.used_pct {
                    let name = win.label.split(' ').next().unwrap_or("Usage");
                    parts.push(format!("{name} {p}%"));
                }
            }
            let state = match a.usage.freshness.phase {
                Freshness::Stale => " · stale",
                Freshness::Failed => " · error",
                Freshness::Refreshing => " · refreshing",
                Freshness::Current => "",
            };
            let full = if parts.is_empty() {
                format!("usage · {}", a.status_word())
            } else {
                format!("{}{state}", parts.join(" · "))
            };
            let compact = parts.first().cloned().unwrap_or("usage".into());
            let tone = match a.usage.worst_status() {
                Some(QuotaStatus::Exhausted) => Tone::Error,
                Some(QuotaStatus::Warning) => Tone::Warning,
                _ => Tone::Secondary,
            };
            right.push(Segment::new(full, tone).clickable(USAGE_CHIP).priority(7));
            let _ = compact;
        }
        right.push(
            Segment::new(truncate_middle(&i.container_id(), 28), Tone::Muted)
                .clickable(CONTAINER_CHIP)
                .priority(6),
        );
        segments::render(area, buf, ctx, &left, &right, t.canvas);
    }
}

impl Screen for CapsuleScreen {
    fn enter(&mut self, _w: &mut World, cx: &mut Cx) {
        cx.focus.focus(PANES);
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(PANES)
    }

    fn animating(&self, _w: &World) -> bool {
        true
    }

    fn is_editing(&self) -> bool {
        false
    }

    fn on_tick(&mut self, w: &mut World, cx: &mut Cx) -> Outcome {
        if let Some(until) = self.prefix_until
            && w.now_ms() >= until
        {
            self.prefix_until = None;
            cx.status("Prefix timed out");
        }
        if self.redraw_flash > 0 {
            self.redraw_flash -= 1;
        }
        if self.pending_spawn.is_some() {
            self.apply_pending_spawn(w, cx);
        }
        // agent output touches the repo state
        let touched = self.daemon(w).map(|d| d.touched_files().len()).unwrap_or(0);
        let now_secs = w.clock.now_secs();
        if let Some(i) = w.instance_mut(&self.instance) {
            i.last_seen_secs = now_secs;
            if touched > 0 {
                i.uncommitted = i.uncommitted.max(touched);
            }
        }
        Outcome::Changed
    }

    fn on_msg(&mut self, msg: &Msg, _w: &mut World, cx: &mut Cx) -> Outcome {
        if let Msg::Takeover { instance, by } = msg
            && *instance == self.instance
        {
            self.takeover = Some(by.clone());
            cx.status(format!("{by} attached to this instance"));
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if let Some(by) = &self.takeover {
            match key.code {
                KeyCode::Enter => {
                    let _ = by;
                    self.takeover = None;
                    if let Some(d) = self.daemon_mut(w) {
                        d.attached_by = Some("this terminal".into());
                    }
                    cx.status("Reconnected · the other client was displaced");
                }
                KeyCode::Esc => {
                    self.takeover = None;
                    cx.go(Go::Detach);
                }
                _ => {}
            }
            return Outcome::Changed;
        }
        if key.ctrl_char('q') {
            self.prefix_until = None;
            self.request_exit(w, cx);
            return Outcome::Changed;
        }
        if key.code == KeyCode::Char('\\') && key.ctrl() {
            self.prefix_until = None;
            self.open_palette(w, cx);
            return Outcome::Changed;
        }
        if self.mode() == Mode::PrefixAwait {
            return self.handle_prefix_cmd(key, w, cx);
        }
        if key.ctrl_char('b') {
            self.enter_prefix(w);
            return Outcome::Changed;
        }
        if key.alt() && key.shift() {
            match key.code {
                KeyCode::Left => {
                    self.resize(Direction::Left, w, cx);
                    return Outcome::Changed;
                }
                KeyCode::Right => {
                    self.resize(Direction::Right, w, cx);
                    return Outcome::Changed;
                }
                KeyCode::Up => {
                    self.resize(Direction::Up, w, cx);
                    return Outcome::Changed;
                }
                KeyCode::Down => {
                    self.resize(Direction::Down, w, cx);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if self.selecting.is_some() {
            return Outcome::Consumed;
        }
        let out = self.forward_key(key, w);
        if key.is_char('y') && w.clipboard.is_some() && out == Outcome::Changed {
            cx.status("Selection copied");
        }
        out
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.takeover.is_some() {
            return Outcome::Consumed;
        }
        self.prefix_until = None;
        if id == MENU {
            self.open_palette(w, cx);
            return Outcome::Changed;
        }
        if id == USAGE_CHIP {
            self.open_usage(w, cx);
            return Outcome::Changed;
        }
        if id == CONTAINER_CHIP {
            self.open_container_info(w, cx);
            return Outcome::Changed;
        }
        if id == CONTEXT {
            self.open_github(w, cx);
            return Outcome::Changed;
        }
        if id == STRIP.sub("left") {
            self.strip_first = self.strip_first.saturating_sub(1);
            return Outcome::Changed;
        }
        if id == STRIP.sub("right") {
            self.strip_first += 1;
            return Outcome::Changed;
        }
        for (i, _) in &self.tab_areas {
            if STRIP.child(*i) == id {
                if let Some(d) = self.daemon_mut(w) {
                    d.active = *i;
                }
                cx.focus.focus(PANES);
                return Outcome::Changed;
            }
        }
        if let Some(pid) = self.pane_at(pos) {
            cx.focus.focus(PANES);
            if let Some(t) = self.daemon_mut(w).and_then(|d| d.active_tab_mut()) {
                t.focused = pid;
            }
            self.prime(pid, w);
            if let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(pid)) {
                pane.term.on_click(pos);
            }
            return Outcome::Changed;
        }
        Outcome::Consumed
    }

    fn on_press(&mut self, _id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        // anchor a text selection on the press so the drag that follows can
        // extend it; the release completes the click
        if let Some(pid) = self.pane_at(pos) {
            cx.focus.focus(PANES);
            if let Some(t) = self.daemon_mut(w).and_then(|d| d.active_tab_mut()) {
                t.focused = pid;
            }
            self.prime(pid, w);
            if let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(pid)) {
                pane.term.on_click(pos);
            }
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_double_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        for (i, _) in &self.tab_areas {
            if STRIP.child(*i) == id {
                let d = self.daemon(w);
                let current = d
                    .and_then(|d| d.tabs.get(*i))
                    .and_then(|t| t.custom_label.clone())
                    .unwrap_or_default();
                let auto = d
                    .and_then(|d| d.tabs.get(*i).map(|t| d.tab_label(t, &|_| None)))
                    .unwrap_or_default();
                let input = TextInput::new(WidgetId::of("capsule.rename.input"), "Label")
                    .value(&current)
                    .placeholder(&auto)
                    .help("Empty restores the automatic name · 16 max")
                    .plain_label();
                let dlg = Dialog::prompt(
                    WidgetId::of("capsule.rename"),
                    "Rename tab",
                    input,
                    "Rename",
                );
                cx.open(Modal::Dialog(dlg), ModalTag::new("rename").n(*i));
                return Outcome::Changed;
            }
        }
        if let Some(pid) = self.pane_at(pos) {
            self.prime(pid, w);
        }
        if let Some(pid) = self.pane_at(pos)
            && let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(pid))
        {
            let o = pane.term.select_word_at(pos);
            if let Some(text) = pane.term.selected_text() {
                w.clipboard = Some(text);
                cx.status("Selection copied");
            }
            return o;
        }
        Outcome::Ignored
    }

    fn on_drag(&mut self, pressed: WidgetId, pos: Position, w: &mut World) -> Outcome {
        // seam drag
        for i in 0..self.seams.len() {
            if PANES.sub("seam").child(i) == pressed {
                let s = self.seams[i].clone();
                self.drag = Some((s.path.clone(), s.container));
                if let Some(t) = self.daemon_mut(w).and_then(|d| d.active_tab_mut())
                    && let Some(PaneNode::Split { split, dir, .. }) = t.root.node_at_mut(&s.path)
                {
                    split.drag_to(*dir, s.container, 1, pos);
                }
                return Outcome::Changed;
            }
        }
        // text selection drag within a pane
        if let Some(pid) = (0..64u64).find(|p| PANES.child(*p as usize) == pressed) {
            self.prime(pid, w);
        }
        if let Some(pid) = (0..64u64).find(|p| PANES.child(*p as usize) == pressed)
            && let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(pid))
        {
            self.selecting = Some(pid);
            return pane.term.on_drag(pos);
        }
        Outcome::Ignored
    }

    fn on_release(
        &mut self,
        _pressed: WidgetId,
        _pos: Position,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        if self.drag.take().is_some() {
            cx.status("Split resized");
            return Outcome::Changed;
        }
        if let Some(pid) = self.selecting.take() {
            if let Some(pane) = self.daemon(w).and_then(|d| d.pane(pid))
                && let Some(text) = pane.term.selected_text()
                && !text.trim().is_empty()
            {
                w.clipboard = Some(text);
                cx.status("Selection copied");
            }
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, pos: Position, w: &mut World) -> Outcome {
        let _ = id;
        if let Some(pid) = self.pane_at(pos) {
            self.prime(pid, w);
        }
        if let Some(pid) = self.pane_at(pos)
            && let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(pid))
        {
            return pane.term.on_wheel(delta);
        }
        Outcome::Ignored
    }

    fn on_paste(&mut self, text: &str, w: &mut World) -> Outcome {
        let now = w.now_ms();
        let ws = self
            .daemon(w)
            .map(|d| d.workspace.clone())
            .unwrap_or_default();
        if let Some(p) = self.focused_pane(w)
            && let Some(pane) = self.daemon_mut(w).and_then(|d| d.pane_mut(p))
        {
            for c in text.chars().filter(|c| *c != '\n' && *c != '\r') {
                pane.type_char(c, now, &ws);
            }
            return Outcome::Changed;
        }
        Outcome::Consumed
    }

    fn picker_items(&mut self, tag: &ModalTag, query: &str, w: &World) -> Option<Vec<PickerItem>> {
        if tag.kind == "palette" {
            return Some(self.palette_items(query, w));
        }
        None
    }

    fn on_modal(
        &mut self,
        tag: &ModalTag,
        result: ModalResult,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        self.dialog_open = false;
        match (tag.kind, result) {
            ("palette", ModalResult::Picked(i)) => {
                if let Some(cmd) = self.palette_cmds.get(i).copied() {
                    self.run_palette(cmd, w, cx);
                }
            }
            ("agent", ModalResult::Picked(i)) => {
                let agent = Agent::ALL.get(i).copied();
                self.continue_spawn(agent, w, cx);
                self.apply_pending_spawn(w, cx);
            }
            ("provider", ModalResult::Picked(i)) => {
                let account = self.picker_accounts.get(i).cloned();
                let agent = self.pending_agent.take();
                if let (Some(a), Some(acc)) = (agent, account.clone()) {
                    // a session choice mutates only session scope
                    let next_id = self.daemon(w).map(|d| d.next_id).unwrap_or(0);
                    w.session_accounts
                        .insert((self.instance.clone(), next_id), acc.clone());
                    self.spawn(Some(a), Some(acc), w, cx);
                    self.apply_pending_spawn(w, cx);
                }
            }
            ("split", ModalResult::Picked(i)) => {
                let intent = match i {
                    0 => Intent::Split(SplitDir::Horizontal, false),
                    1 => Intent::Split(SplitDir::Horizontal, true),
                    2 => Intent::Split(SplitDir::Vertical, false),
                    _ => Intent::Split(SplitDir::Vertical, true),
                };
                self.spawn_flow(intent, w, cx);
            }
            ("closetarget", ModalResult::Picked(i)) => self.confirm_close(i == 1, w, cx),
            (
                "closepane",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                if let Some(p) = self.focused_pane(w)
                    && let Some(d) = self.daemon_mut(w)
                {
                    let tab_closed = d.close_pane(p);
                    let empty = d.tabs.is_empty();
                    crate::domain::fixtures::refresh_snapshots(w);
                    if empty {
                        cx.status("Last session ended · the Capsule shuts down");
                        self.end_instance(false, cx);
                    } else {
                        cx.status(if tab_closed {
                            "Tab closed"
                        } else {
                            "Pane closed"
                        });
                    }
                }
            }
            (
                "closetab",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                if let Some(d) = self.daemon_mut(w) {
                    let i = d.active;
                    d.close_tab(i);
                    let empty = d.tabs.is_empty();
                    crate::domain::fixtures::refresh_snapshots(w);
                    if empty {
                        cx.status("Last session ended · the Capsule shuts down");
                        self.end_instance(false, cx);
                    } else {
                        cx.status("Tab closed");
                    }
                }
            }
            (
                "exit",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                self.end_instance(false, cx);
            }
            ("exitdirty", ModalResult::Choice(Some(row))) => match row {
                0 => self.spawn_flow(Intent::NewTab, w, cx),
                1 => {
                    let touched = self
                        .daemon(w)
                        .map(|d| d.touched_files())
                        .unwrap_or_default();
                    let ws = self
                        .daemon(w)
                        .map(|d| d.workspace.clone())
                        .unwrap_or_default();
                    let inst = w.instance(&self.instance);
                    let mut props = vec![
                        Prop::new(
                            ws.as_str(),
                            format!(
                                "{} changed · {} unpushed",
                                inst.map(|i| i.uncommitted).unwrap_or(0) + touched.len(),
                                inst.map(|i| i.unpushed).unwrap_or(0)
                            ),
                        )
                        .tone(Tone::Warning),
                    ];
                    props.push(Prop::new("M", "src/settlement/retry.rs"));
                    props.push(Prop::new("M", "src/settlement/mod.rs"));
                    for f in touched {
                        props.push(Prop::new("A", f));
                    }
                    if inst.map(|i| i.unpushed).unwrap_or(0) > 0 {
                        props.push(
                            Prop::new(
                                "↑",
                                format!(
                                    "{} commit not pushed",
                                    inst.map(|i| i.unpushed).unwrap_or(0)
                                ),
                            )
                            .tone(Tone::Warning),
                        );
                    }
                    let d =
                        InfoDialog::new(WidgetId::of("capsule.inspect"), "Inspect changes", props)
                            .meta("Esc returns to the exit choices");
                    cx.open(Modal::Info(d), ModalTag::new("inspect"));
                }
                2 => {
                    cx.status("Exited · changes kept in the preserved instance");
                    self.end_instance(false, cx);
                }
                _ => {
                    let ws = self
                        .daemon(w)
                        .map(|d| d.workspace.clone())
                        .unwrap_or_default();
                    let facts = vec![
                        Prop::new(
                            "Action",
                            "discard every uncommitted change and unpushed commit",
                        )
                        .tone(Tone::Error),
                        Prop::new(
                            "Target",
                            format!(
                                "{ws} · instance {}",
                                self.instance.trim_start_matches("jk-")
                            ),
                        ),
                        Prop::new("Reversible", "no").tone(Tone::Secondary),
                    ];
                    let d = Dialog::facts(
                        WidgetId::of("capsule.discard"),
                        "Exit and discard changes?",
                        facts,
                        vec![],
                        Some(&ws),
                        Button::danger(WidgetId::of("capsule.discard").sub("ok"), "Discard"),
                    );
                    cx.open(Modal::Dialog(d), ModalTag::new("discard"));
                }
            },
            ("inspect", _) => {
                self.request_exit(w, cx);
            }
            (
                "discard",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                if let Some(i) = w.instance_mut(&self.instance) {
                    i.uncommitted = 0;
                    i.unpushed = 0;
                }
                cx.status("Changes discarded");
                self.end_instance(false, cx);
            }
            (
                "rename",
                ModalResult::Dialog {
                    action: Some(1),
                    text,
                },
            ) => {
                let label: String = text.unwrap_or_default().chars().take(MAX_LABEL).collect();
                if let Some(t) = self.daemon_mut(w).and_then(|d| d.tabs.get_mut(tag.n)) {
                    t.custom_label = if label.trim().is_empty() {
                        None
                    } else {
                        Some(label.clone())
                    };
                }
                cx.status(if label.trim().is_empty() {
                    "Tab name reset"
                } else {
                    "Tab renamed"
                });
            }
            (
                "export",
                ModalResult::Dialog {
                    action: Some(1),
                    text,
                },
            ) => {
                let p = text.unwrap_or_default();
                self.finish_export(&p, cx);
            }
            ("info", ModalResult::Info(InfoResult::Copy(v)))
            | ("github", ModalResult::Info(InfoResult::Copy(v))) => {
                cx.copy(v);
            }
            ("github", ModalResult::Info(InfoResult::Action(0))) => {
                cx.status("Opened the pull request on the host (gated host action)");
            }
            ("usage", _) => {}
            (_, ModalResult::Cancelled)
            | (
                _,
                ModalResult::Dialog {
                    action: Some(0), ..
                },
            )
            | (_, ModalResult::Choice(None)) => {
                self.pending_intent = None;
                self.pending_agent = None;
            }
            _ => {}
        }
        cx.focus.focus(PANES);
        Outcome::Changed
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        fill(buf, area, t.base());
        if let Some(by) = &self.takeover {
            let lines = [
                ("Attach taken over", t.title()),
                ("", t.base()),
                (
                    &*format!(
                        "{by} attached to {} {}.",
                        self.daemon(w)
                            .map(|d| d.workspace.as_str())
                            .unwrap_or("this instance"),
                        w.clock.ago(w.now_secs() - 3)
                    ),
                    t.secondary(),
                ),
                ("The instance, tabs and panes keep running.", t.secondary()),
                ("", t.base()),
                (
                    "Enter Reconnect (take it back)   Esc Workspace manager",
                    t.muted(),
                ),
            ];
            let y0 = area.y + area.height.saturating_sub(lines.len() as u16) / 2;
            for (i, (text, style)) in lines.iter().enumerate() {
                let w_ = width(text) as u16;
                let x = area.x + area.width.saturating_sub(w_) / 2;
                buf.set_string(x, y0 + i as u16, text, style.bg(t.canvas));
            }
            return;
        }
        let strip = Rect::new(area.x, area.y, area.width, 2);
        let context = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(3),
        );
        self.body = body;
        self.draw_strip(strip, buf, ctx, w);
        self.draw_panes(body, buf, ctx, w);
        self.draw_context(context, buf, ctx, w);
        if self.redraw_flash > 0 {
            buf.set_string(
                area.right().saturating_sub(10),
                area.y,
                " redrawn ",
                t.faint().bg(t.canvas),
            );
        }
    }

    fn hints(&self, _focus: Option<WidgetId>, w: &World) -> Vec<Hint> {
        match self.mode() {
            Mode::PrefixAwait => vec![
                hint("c", "New tab"),
                hint("n p", "Tabs"),
                hint("x", "Close"),
                hint("h j k l", "Nav"),
                hint("\"", "Split ↕"),
                hint("%", "Split ↔"),
                hint("z", "Zoom"),
                hint("&", "Kill tab"),
                hint("Ctrl+L", "Clear"),
                hint("d", "Detach"),
                hint("u", "Usage"),
                hint("Space", "Palette"),
            ],
            Mode::Select => vec![hint("drag", "Select"), hint("release", "Copy")],
            Mode::Drag => vec![hint("drag", "Resize"), hint("release", "Done")],
            _ => {
                let scrolled = self
                    .focused_pane(w)
                    .and_then(|p| self.daemon(w).and_then(|d| d.pane(p)))
                    .is_some_and(|p| !p.term.follow);
                if scrolled {
                    vec![
                        hint("↑↓", "Scroll"),
                        hint("PgUp PgDn", "Page"),
                        hint("End", "Live"),
                        hint("Esc", "Exit scrollback"),
                        hint("Ctrl+\\", "Menu"),
                    ]
                } else {
                    vec![
                        hint("Ctrl+B", "Prefix"),
                        hint("Ctrl+\\", "Menu"),
                        hint("Alt+Shift+↑↓←→", "Resize"),
                        hint("click", "Focus pane"),
                        hint("Ctrl+Q", "Quit"),
                    ]
                }
            }
        }
    }

    fn crumb(&self, w: &World) -> String {
        let d = self.daemon(w);
        let ws = d.map(|d| d.workspace.as_str()).unwrap_or("instance");
        match d.and_then(|d| d.active_tab().map(|t| d.tab_label(t, &|_| None))) {
            Some(tab) => format!("Capsule › {ws} › {tab}"),
            None => format!("Capsule › {ws}"),
        }
    }

    fn strip_right(&self, w: &World) -> Vec<Segment> {
        let n = w.running_count();
        vec![Segment::new(plural(n, "instance", "instances"), Tone::Muted).priority(4)]
    }

    fn on_esc_top(&mut self, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Consumed
    }
}

// ------------------------------------------------------------- usage dialog

pub struct UsageDialog {
    pub accounts: Vec<AccountId>,
    pub tab: usize,
    pub tabbar_focus: bool,
    pub scroll: junie_tui::core::scroll::ScrollState,
    pub refreshing_until: Option<i64>,
    pub result: Option<ModalResult>,
    pub area: Rect,
    pub tab_areas: Vec<Rect>,
}

impl UsageDialog {
    pub fn new(accounts: Vec<AccountId>, _w: &World) -> Self {
        Self {
            accounts,
            tab: 0,
            tabbar_focus: true,
            scroll: junie_tui::core::scroll::ScrollState::default(),
            refreshing_until: None,
            result: None,
            area: Rect::ZERO,
            tab_areas: vec![],
        }
    }

    fn meter_tone(win: &crate::domain::usage::QuotaWindow, fresh: Freshness) -> MeterTone {
        if fresh != Freshness::Current {
            return MeterTone::Stale;
        }
        match win.status {
            QuotaStatus::Exhausted => MeterTone::Exhausted,
            QuotaStatus::Warning => MeterTone::Warning,
            _ => MeterTone::Normal,
        }
    }
}

impl CustomModal for UsageDialog {
    fn on_key(&mut self, key: &Key, _focus: &mut Focus, _ring: &FocusRing, w: &World) -> Outcome {
        let n = self.accounts.len() + 1;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.tabbar_focus {
                    self.result = Some(ModalResult::Custom("close".into()));
                } else {
                    self.tabbar_focus = true;
                }
                Outcome::Changed
            }
            KeyCode::Tab => {
                self.tabbar_focus = !self.tabbar_focus;
                Outcome::Changed
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.tab = (self.tab + n - 1) % n;
                self.scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.tab = (self.tab + 1) % n;
                self.scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll.scroll_by(-1);
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.scroll.page_down();
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.scroll.page_up();
                Outcome::Changed
            }
            KeyCode::Char('r') => {
                if self.refreshing_until.is_none() {
                    self.refreshing_until = Some(w.now_ms() + 800);
                }
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    fn on_click(
        &mut self,
        id: WidgetId,
        _pos: Position,
        _focus: &mut Focus,
        _w: &World,
    ) -> Outcome {
        for (i, _) in self.tab_areas.iter().enumerate() {
            if WidgetId::of("capsule.usage.tab").child(i) == id {
                self.tab = i;
                self.tabbar_focus = true;
                return Outcome::Changed;
            }
        }
        Outcome::Consumed
    }

    fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    fn on_tick(&mut self, w: &World) -> Outcome {
        if let Some(u) = self.refreshing_until
            && w.now_ms() >= u
        {
            self.refreshing_until = None;
            return Outcome::Changed;
        }
        if self.refreshing_until.is_some() {
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn done(&mut self) -> Option<ModalResult> {
        self.result.take()
    }

    fn initial_focus(&self) -> WidgetId {
        WidgetId::of("capsule.usage")
    }

    fn hints(&self) -> Vec<Hint> {
        vec![
            hint("r", "Refresh"),
            hint("Tab", "Tabs / body"),
            hint("← →", "Provider"),
            hint("Esc", "Close"),
        ]
    }

    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let wdt = 66u16.min(screen.width.saturating_sub(4));
        let h = screen.height.saturating_sub(4).min(24);
        let (area, inner) = modal_frame(
            screen,
            buf,
            ctx,
            wdt,
            h,
            "Usage",
            Some("read-only projection"),
            false,
        );
        self.area = area;
        let t: &Theme = ctx.theme;
        let bg = t.surface_elevated;
        // tab row
        let mut labels = vec!["Overview".to_owned()];
        for id in &self.accounts {
            if let Some(a) = w.accounts.get(id) {
                labels.push(format!(
                    "{} · {}",
                    a.agent.map(|x| x.label()).unwrap_or(a.provider.short()),
                    a.display_name
                ));
            }
        }
        self.tab_areas.clear();
        let mut x = inner.x;
        let y = inner.y;
        for xx in inner.left()..inner.right() {
            buf.set_string(xx, y + 1, "─", Style::new().fg(t.border_subtle).bg(bg));
        }
        for (i, l) in labels.iter().enumerate() {
            let lw = width(l) as u16 + 2;
            if x + lw > inner.right() {
                break;
            }
            let active = i == self.tab;
            let mut st = Style::new().bg(bg).fg(if active {
                t.text_primary
            } else {
                t.text_secondary
            });
            if active {
                st = st.add_modifier(Modifier::BOLD);
            }
            if active && self.tabbar_focus {
                buf.set_string(x, y, "▎", Style::new().fg(t.focus).bg(bg));
            }
            buf.set_string(x + 1, y, l, st);
            if active {
                for xx in x + 1..x + lw - 1 {
                    buf.set_string(xx, y + 1, "━", Style::new().fg(t.border_strong).bg(bg));
                }
            }
            let r = Rect::new(x, y, lw, 1);
            ctx.clickable(WidgetId::of("capsule.usage.tab").child(i), r);
            self.tab_areas.push(r);
            x += lw + 1;
        }
        let body = Rect::new(
            inner.x,
            y + 3,
            inner.width,
            inner.bottom().saturating_sub(y + 5),
        );
        // build body lines
        enum Line {
            Text(String, Tone),
            Meter(String, u8, String, MeterTone),
        }
        let mut lines: Vec<Line> = vec![];
        let refreshing = self.refreshing_until.is_some();
        if self.tab == 0 {
            for id in &self.accounts {
                if let Some(a) = w.accounts.get(id) {
                    let name = format!(
                        "{} · {}",
                        a.agent.map(|x| x.label()).unwrap_or(a.provider.short()),
                        a.display_name
                    );
                    let state = if refreshing {
                        "refreshing…".to_owned()
                    } else {
                        a.usage.freshness.phase.label().to_owned()
                    };
                    match a.usage.windows.iter().find(|w| w.used_pct.is_some()) {
                        Some(win) => lines.push(Line::Meter(
                            name,
                            win.used_pct.unwrap_or(0),
                            format!("{} · {}", win.label, state),
                            Self::meter_tone(
                                win,
                                if refreshing {
                                    Freshness::Refreshing
                                } else {
                                    a.usage.freshness.phase
                                },
                            ),
                        )),
                        None => lines.push(Line::Text(
                            format!("{name}   {}", a.status_word()),
                            Tone::Muted,
                        )),
                    }
                }
            }
            if lines.is_empty() {
                lines.push(Line::Text(
                    "Providers   usage unavailable".into(),
                    Tone::Muted,
                ));
            }
        } else if let Some(a) = self
            .accounts
            .get(self.tab - 1)
            .and_then(|id| w.accounts.get(id))
        {
            lines.push(Line::Text(
                format!("Identity provider   {}", a.provider.label()),
                Tone::Normal,
            ));
            lines.push(Line::Text(
                format!(
                    "Identity account    {} ({})",
                    a.display_name,
                    a.identity.label()
                ),
                Tone::Normal,
            ));
            lines.push(Line::Text(
                format!(
                    "Identity activity   last request {}",
                    a.last_refresh_secs
                        .map(|s| w.clock.ago(s))
                        .unwrap_or("never".into())
                ),
                Tone::Normal,
            ));
            lines.push(Line::Text(String::new(), Tone::Normal));
            for win in &a.usage.windows {
                if win.has_meter() {
                    let reset = win
                        .reset_secs
                        .map(|r| w.clock.reset_label(r))
                        .unwrap_or_default();
                    lines.push(Line::Meter(
                        win.label.clone(),
                        win.used_pct.unwrap_or(0),
                        format!("{} · {reset}", win.value_label()),
                        Self::meter_tone(
                            win,
                            if refreshing {
                                Freshness::Refreshing
                            } else {
                                a.usage.freshness.phase
                            },
                        ),
                    ));
                } else {
                    lines.push(Line::Text(
                        format!("{}   {}", win.label, win.value_label()),
                        Tone::Muted,
                    ));
                }
            }
            lines.push(Line::Text(String::new(), Tone::Normal));
            let fresh = if refreshing {
                format!("{} refreshing…", spinner_frame(w.now_ms() as u64 / 80))
            } else {
                match a.usage.freshness.phase {
                    Freshness::Current => format!(
                        "current · refreshed {}",
                        a.last_refresh_secs
                            .map(|s| w.clock.ago(s))
                            .unwrap_or("just now".into())
                    ),
                    Freshness::Stale => format!(
                        "stale · refreshed {}",
                        a.last_refresh_secs
                            .map(|s| w.clock.ago(s))
                            .unwrap_or("?".into())
                    ),
                    Freshness::Failed => format!(
                        "! error: {}",
                        a.issue
                            .as_ref()
                            .map(|i| i.message.clone())
                            .unwrap_or("refresh failed".into())
                    ),
                    Freshness::Refreshing => "refreshing…".into(),
                }
            };
            lines.push(Line::Text(
                fresh,
                if a.usage.freshness.phase == Freshness::Failed && !refreshing {
                    Tone::Error
                } else {
                    Tone::Muted
                },
            ));
        }
        self.scroll.set_content(lines.len());
        self.scroll.set_viewport(body.height as usize);
        ctx.scrollable(WidgetId::of("capsule.usage"), body);
        let narrow = inner.width < 60;
        for (k, i) in self.scroll.visible_range().enumerate() {
            let yy = body.y + k as u16;
            match &lines[i] {
                Line::Text(s, tone) => buf.set_string(
                    body.x,
                    yy,
                    truncate(s, body.width as usize),
                    Style::new().fg(t.tone(*tone)).bg(bg),
                ),
                Line::Meter(label, pct, value, tone) => {
                    let lw = if narrow { body.width.min(20) } else { 20 };
                    buf.set_string(body.x, yy, truncate(label, lw as usize), t.primary().bg(bg));
                    let mx = body.x + lw + 1;
                    if body.width > lw + 12 {
                        render_meter(
                            Rect::new(mx, yy, body.right().saturating_sub(mx), 1),
                            buf,
                            ctx,
                            *pct,
                            &format!("{pct}%"),
                            *tone,
                            bg,
                        );
                    }
                    let _ = value;
                }
            }
        }
        if self.scroll.overflows() {
            scrollbar::render_vertical(
                Rect::new(area.right() - 2, body.y, 1, body.height),
                buf,
                ctx,
                WidgetId::of("capsule.usage"),
                &self.scroll,
                true,
            );
        }
        let hint = "r refresh  Tab tabs/body  ←→ provider  Esc close";
        buf.set_string(
            inner.x,
            inner.bottom().saturating_sub(1),
            truncate(hint, inner.width as usize),
            t.faint().bg(bg),
        );
        let _ = props::render;
    }
}
