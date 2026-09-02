//! Workspace Editor: General · Mounts · Roles · Environments · Auth, with a
//! separate original and pending state, `• N changes` in the strip, a save
//! preview and an asynchronous save that returns to the manager.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{fit, truncate, width};
use junie_tui::widgets::button::{Button, row_layout_right};
use junie_tui::widgets::choice::Checkbox;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::input::{InputEvent, TextInput};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::picker::{Picker, PickerItem};
use junie_tui::widgets::progress::spinner_frame;
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::segments::Segment;
use junie_tui::widgets::select::Select;
use junie_tui::widgets::tabs::{TabEvent, TabItem, Tabs};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;

use super::config::{ConfigTabs, Doc, Scope, Tab as CfgTab};
use super::modals::InfoDialog;
use super::{Cx, Go, Modal, ModalResult, ModalTag, Screen, plural};
use crate::domain::workspace::{
    AllowedRoles, DirtyExitPolicy, RoleEntry, RoleName, RoleSource, Workspace, WorkspaceId,
};
use crate::sim::world::{Msg, World};

pub const TABS: WidgetId = WidgetId::of("editor.tabs");
pub const NAME: WidgetId = WidgetId::of("editor.name");
pub const WORKDIR: WidgetId = WidgetId::of("editor.workdir");
pub const KEEP_AWAKE: WidgetId = WidgetId::of("editor.keep_awake");
pub const GIT_PULL: WidgetId = WidgetId::of("editor.git_pull");
pub const DIRTY_POLICY: WidgetId = WidgetId::of("editor.dirty_policy");
pub const ROLES: WidgetId = WidgetId::of("editor.roles");
pub const CANCEL: WidgetId = WidgetId::of("editor.cancel");
pub const SAVE: WidgetId = WidgetId::of("editor.save");
const BASE: WidgetId = WidgetId::of("editor.cfg");

const TAB_NAMES: [&str; 5] = ["General", "Mounts", "Roles", "Environments", "Auth"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdTab {
    General,
    Mounts,
    Roles,
    Environments,
    Auth,
}

impl EdTab {
    const ALL: [EdTab; 5] = [
        EdTab::General,
        EdTab::Mounts,
        EdTab::Roles,
        EdTab::Environments,
        EdTab::Auth,
    ];
    fn cfg(self) -> Option<CfgTab> {
        match self {
            EdTab::Mounts => Some(CfgTab::Mounts),
            EdTab::Environments => Some(CfgTab::Environments),
            EdTab::Auth => Some(CfgTab::Auth),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RoleRow {
    Role(String),
    Load,
}

pub struct EditorScreen {
    pub workspace: Option<WorkspaceId>,
    pub original: Workspace,
    pub pending: Workspace,
    pub cfg: ConfigTabs,
    tabs: Tabs,
    tab: EdTab,
    name: TextInput,
    workdir_button: Button,
    keep_awake: Checkbox,
    git_pull: Checkbox,
    dirty_policy: Select,
    role_rows: Vec<RoleRow>,
    role_cursor: usize,
    role_area: Rect,
    cancel: Button,
    save: Button,
    saving: bool,
    save_error: Option<String>,
    pending_load: Option<String>,
    picker_targets: Vec<String>,
    row_status: Option<String>,
    /// Names of the other workspaces, for uniqueness checks.
    taken: Vec<(String, WorkspaceId)>,
    taken_roles: Vec<RoleName>,
}

/// Roles that get their own Environments / Auth section: the allowed set
/// (explicit list, or the whole registry when all roles are allowed) plus
/// the default.
fn allowed_sections(ws: &Workspace, registry: &[RoleName]) -> Vec<RoleName> {
    let mut v: Vec<RoleName> = match &ws.roles.allowed {
        AllowedRoles::All => registry.to_vec(),
        AllowedRoles::Custom(l) => l.clone(),
    };
    if let Some(d) = &ws.roles.default
        && !v.contains(d)
    {
        v.push(d.clone());
    }
    v
}

impl EditorScreen {
    pub fn new(w: &World, workspace: Option<WorkspaceId>, pending: Option<Workspace>) -> Self {
        let create = pending.is_some() && workspace.is_none();
        let original = workspace
            .and_then(|id| w.workspace(id))
            .cloned()
            .unwrap_or_else(|| {
                let mut o = pending
                    .clone()
                    .unwrap_or_else(|| Workspace::new(0, "new", "/workspace"));
                // a freshly created Workspace starts one change away from "nothing"
                o.name = String::new();
                o
            });
        let pending = pending
            .or_else(|| workspace.and_then(|id| w.workspace(id)).cloned())
            .unwrap_or_else(|| Workspace::new(0, "new", "/workspace"));
        let registry: Vec<RoleName> = w.roles.iter().map(|r| r.name.clone()).collect();
        let sections = allowed_sections(&pending, &registry);
        let cfg = ConfigTabs::new(
            Scope::Workspace,
            Doc::from_workspace(&original),
            Doc::from_workspace(&pending),
            BASE,
            sections,
            registry,
        );
        let policy_idx = match pending.dirty_policy {
            DirtyExitPolicy::Ask => 0,
            DirtyExitPolicy::Keep => 1,
            DirtyExitPolicy::Discard => 2,
        };
        let mut s = Self {
            workspace,
            name: TextInput::new(NAME, "Name")
                .required(true)
                .help("Directory basename by default")
                .value(&pending.name)
                .validator(|v| {
                    let v = v.trim();
                    if v.is_empty() {
                        Some("Name is required".into())
                    } else if v.contains('/') {
                        Some("Name cannot contain /".into())
                    } else {
                        None
                    }
                }),
            workdir_button: Button::secondary(WORKDIR, "Choose…"),
            keep_awake: Checkbox::new(KEEP_AWAKE, "Keep awake", pending.keep_awake),
            git_pull: Checkbox::new(GIT_PULL, "Git pull before launch", pending.git_pull),
            dirty_policy: Select::new(
                DIRTY_POLICY,
                "On dirty exit",
                &[
                    "ask · show the exit dialog",
                    "keep · preserve changes silently",
                    "discard · drop changes silently",
                ],
                policy_idx,
            ),
            original,
            pending,
            cfg,
            tabs: Tabs::new(TABS, &TAB_NAMES),
            tab: EdTab::General,
            role_rows: vec![],
            role_cursor: 0,
            role_area: Rect::ZERO,
            cancel: Button::subtle(CANCEL, "Cancel"),
            save: Button::primary(SAVE, "Save…"),
            saving: false,
            save_error: None,
            pending_load: None,
            picker_targets: vec![],
            row_status: None,
            taken: vec![],
            taken_roles: w.roles.iter().map(|r| r.name.clone()).collect(),
        };
        let _ = create;
        s.build_roles(w);
        s
    }

    pub fn is_create(&self) -> bool {
        self.workspace.is_none()
    }

    fn sync_pending(&mut self) {
        let registry: Vec<RoleName> = self.taken_roles.clone();
        self.cfg
            .set_sections(allowed_sections(&self.pending, &registry));
        self.pending.name = self.name.text().trim().to_owned();
        self.pending.keep_awake = self.keep_awake.checked;
        self.pending.git_pull = self.git_pull.checked;
        self.pending.dirty_policy = match self.dirty_policy.selected {
            1 => DirtyExitPolicy::Keep,
            2 => DirtyExitPolicy::Discard,
            _ => DirtyExitPolicy::Ask,
        };
        self.cfg.pending.apply_to_workspace(&mut self.pending);
    }

    pub fn change_count(&self) -> usize {
        self.pending.change_count(&self.original)
    }

    fn general_dirty(&self) -> bool {
        self.pending.name != self.original.name
            || self.pending.workdir != self.original.workdir
            || self.pending.keep_awake != self.original.keep_awake
            || self.pending.git_pull != self.original.git_pull
            || self.pending.dirty_policy != self.original.dirty_policy
    }

    fn roles_dirty(&self) -> bool {
        self.pending.roles != self.original.roles
    }

    fn build_roles(&mut self, w: &World) {
        let mut rows: Vec<RoleRow> = w
            .roles
            .iter()
            .map(|r| RoleRow::Role(r.name.clone()))
            .collect();
        if let AllowedRoles::Custom(list) = &self.pending.roles.allowed {
            for r in list {
                if !rows.iter().any(|x| matches!(x, RoleRow::Role(n) if n == r)) {
                    rows.push(RoleRow::Role(r.clone()));
                }
            }
        }
        if let Some(d) = &self.pending.roles.default
            && !rows.iter().any(|x| matches!(x, RoleRow::Role(n) if n == d))
        {
            rows.push(RoleRow::Role(d.clone()));
        }
        rows.push(RoleRow::Load);
        self.role_rows = rows;
        if self.role_cursor >= self.role_rows.len() {
            self.role_cursor = self.role_rows.len() - 1;
        }
    }

    fn set_tab(&mut self, i: usize, cx: &mut Cx) {
        self.tab = EdTab::ALL[i.min(4)];
        self.tabs.set_active(i);
        cx.focus.focus(TABS);
    }

    fn body_focus(&self) -> WidgetId {
        match self.tab {
            EdTab::General => NAME,
            EdTab::Roles => ROLES,
            t => self.cfg.list_id(t.cfg().unwrap()),
        }
    }

    // ------------------------------------------------------------- save

    fn validate(&mut self, cx: &mut Cx) -> bool {
        if !self.name.validate() {
            self.set_tab(0, cx);
            cx.focus.focus(NAME);
            cx.error("Name is required and cannot contain /");
            return false;
        }
        let name = self.name.text().trim().to_owned();
        if let Some(other) = self.taken_by(&name) {
            self.name.error = Some(format!("A workspace named {name} already exists"));
            self.set_tab(0, cx);
            cx.focus.focus(NAME);
            cx.error(format!(
                "A workspace named {name} already exists (#{other})"
            ));
            return false;
        }
        if self.cfg.tab_error(CfgTab::Environments) {
            self.set_tab(3, cx);
            cx.error("An environment key is invalid");
            return false;
        }
        true
    }

    fn taken_by(&self, name: &str) -> Option<WorkspaceId> {
        self.taken
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, id)| *id)
    }

    fn open_preview(&mut self, w: &World, cx: &mut Cx) {
        self.sync_pending();
        if !self.validate(cx) {
            return;
        }
        let n = self.change_count();
        if n == 0 && !self.is_create() {
            cx.status("Nothing to save");
            return;
        }
        let blockers = self.cfg.blockers();
        let mut facts = vec![
            Prop::new(
                "Workspace",
                if self.is_create() {
                    format!("{} · new", self.pending.name)
                } else {
                    self.pending.name.clone()
                },
            ),
            Prop::new(
                "Scope",
                format!(
                    "workspace config · {}",
                    w.tilde(&format!(
                        "{}/.jackin/workspaces/{}.toml",
                        w.home, self.pending.name
                    ))
                ),
            ),
            Prop::new("Changes", plural(n.max(1), "change", "changes")),
        ];
        if self.pending.name != self.original.name && !self.is_create() {
            facts.push(Prop::new(
                "Name",
                format!("{} → {}", self.original.name, self.pending.name),
            ));
        }
        if self.pending.workdir != self.original.workdir {
            facts.push(Prop::new(
                "Working dir",
                format!("{} → {}", self.original.workdir, self.pending.workdir),
            ));
        }
        facts.extend(self.cfg.summary_facts());
        if self.pending.roles != self.original.roles {
            facts.push(Prop::new(
                "Roles",
                format!(
                    "{} · default {}",
                    self.pending.roles.summary().to_lowercase(),
                    self.pending.roles.default.clone().unwrap_or("none".into())
                ),
            ));
        }
        for b in &blockers {
            facts.push(Prop::new("Blocker", b.clone()).tone(Tone::Error));
        }
        let mut code = self.cfg.diff_lines();
        if self.is_create() {
            code.insert(
                0,
                format!(
                    "+ workspace {}  {}",
                    self.pending.name, self.pending.workdir
                ),
            );
        }
        if self.pending.keep_awake != self.original.keep_awake {
            code.push(format!(
                "~ keep_awake {} → {}",
                self.original.keep_awake, self.pending.keep_awake
            ));
        }
        if self.pending.git_pull != self.original.git_pull {
            code.push(format!(
                "~ git_pull {} → {}",
                self.original.git_pull, self.pending.git_pull
            ));
        }
        if self.pending.dirty_policy != self.original.dirty_policy {
            code.push(format!(
                "~ dirty_exit {} → {}",
                self.original.dirty_policy.label(),
                self.pending.dirty_policy.label()
            ));
        }
        let confirm = if blockers.is_empty() {
            Button::primary(WidgetId::of("editor.preview.save"), "Save")
        } else {
            Button::primary(WidgetId::of("editor.preview.cleanup"), "Clean up & save…")
        };
        let title = if self.is_create() {
            "Create workspace"
        } else {
            "Save workspace"
        };
        let d = Dialog::facts(
            WidgetId::of("editor.preview"),
            title,
            facts,
            code,
            None,
            confirm,
        );
        cx.open(
            Modal::Dialog(d),
            ModalTag::new(if blockers.is_empty() {
                "preview"
            } else {
                "preview.cleanup"
            }),
        );
    }

    fn start_save(&mut self, w: &mut World, cx: &mut Cx) {
        self.sync_pending();
        self.saving = true;
        self.save.busy = true;
        let ok = !w.save_fails_once;
        w.save_fails_once = false;
        let id = self.workspace.unwrap_or(w.next_workspace_id);
        w.schedule(900, Msg::WorkspaceSaved { id, ok });
        cx.status(format!("Saving {}…", self.pending.name));
        cx.focus.focus(SAVE);
    }

    fn finish_save(&mut self, ok: bool, w: &mut World, cx: &mut Cx) {
        self.saving = false;
        self.save.busy = false;
        if !ok {
            let title = format!("Save failed · {}", self.pending.name);
            let d = InfoDialog::new(
                WidgetId::of("editor.savefail"),
                &title,
                vec![
                    Prop::new(
                        "Error",
                        "write failed: ~/.jackin/workspaces is not writable (EACCES)",
                    ),
                    Prop::new("State", "your edits are intact · nothing was written"),
                    Prop::new("Next", "fix the permission and Save again"),
                ],
            )
            .error()
            .width(66);
            cx.open(Modal::Info(d), ModalTag::new("savefail"));
            return;
        }
        let name = self.pending.name.clone();
        match self.workspace {
            Some(id) => {
                if let Some(ws) = w.workspace_mut(id) {
                    *ws = self.pending.clone();
                    ws.id = id;
                }
            }
            None => {
                let mut ws = self.pending.clone();
                ws.id = w.next_workspace_id;
                w.next_workspace_id += 1;
                self.workspace = Some(ws.id);
                w.workspaces.push(ws);
            }
        }
        self.original = self.pending.clone();
        cx.status(format!("Workspace {name} saved"));
        cx.go(Go::Manager);
    }

    fn leave(&mut self, cx: &mut Cx) {
        self.sync_pending();
        if self.change_count() > 0 || self.is_create() {
            let d = Dialog::confirm(
                WidgetId::of("editor.exit"),
                "Unsaved changes",
                &format!(
                    "Save changes before leaving? {} would be lost.",
                    plural(self.change_count().max(1), "change", "changes")
                ),
                "Save",
            )
            .with_actions(
                vec![
                    Button::subtle(WidgetId::of("editor.exit.cancel"), "Cancel"),
                    Button::danger(WidgetId::of("editor.exit.discard"), "Discard"),
                    Button::primary(WidgetId::of("editor.exit.save"), "Save"),
                ],
                Some(0),
            );
            cx.open(Modal::Dialog(d), ModalTag::new("exit"));
        } else {
            cx.go(Go::Manager);
        }
    }

    // ----------------------------------------------------------- general

    fn render_general(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        let x = area.x + 2;
        let mut y = area.y;
        let fw = area.width.saturating_sub(4).min(72);
        self.name.render(Rect::new(x, y, fw, 3), buf, ctx, bg);
        y += 4;
        buf.set_string(
            x,
            y,
            "Working directory *",
            t.secondary().bg(bg).add_modifier(Modifier::BOLD),
        );
        y += 1;
        let wd_focused = ctx.interaction.focused(WORKDIR);
        buf.set_string(
            x,
            y,
            fit(
                &truncate(&self.pending.workdir, fw.saturating_sub(14) as usize),
                fw.saturating_sub(14) as usize,
            ),
            if wd_focused {
                t.primary().bg(bg)
            } else {
                t.secondary().bg(bg)
            },
        );
        self.workdir_button
            .render(Rect::new(x + fw.saturating_sub(11), y, 11, 1), buf, ctx, bg);
        y += 1;
        let hint = if self.pending.workdir != self.original.workdir {
            format!("Inside the Construct · was {}", self.original.workdir)
        } else {
            "Inside the Construct".into()
        };
        buf.set_string(x, y, truncate(&hint, fw as usize), t.faint().bg(bg));
        y += 2;
        self.keep_awake.render(Rect::new(x, y, 30, 1), buf, ctx, bg);
        buf.set_string(x + 30, y, "macOS only", t.faint().bg(bg));
        y += 1;
        self.git_pull.render(Rect::new(x, y, 30, 1), buf, ctx, bg);
        y += 2;
        self.dirty_policy
            .render(Rect::new(x, y, fw.min(48), 3), buf, ctx, bg);
        let _ = w;
    }

    fn render_roles(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        self.build_roles(w);
        let t = ctx.theme;
        let bg = t.canvas;
        let focused = ctx.interaction.focused(ROLES);
        let (allowed_n, total) = match &self.pending.roles.allowed {
            AllowedRoles::All => (w.roles.len(), w.roles.len()),
            AllowedRoles::Custom(list) => (list.len(), w.roles.len()),
        };
        let head = format!(
            "Allowed roles  {}",
            if matches!(self.pending.roles.allowed, AllowedRoles::All) {
                "all".to_owned()
            } else {
                format!("{allowed_n} of {total}")
            }
        );
        buf.set_string(
            area.x + 2,
            area.y,
            &head,
            t.secondary().bg(bg).add_modifier(Modifier::BOLD),
        );
        let def = format!(
            "default {}",
            self.pending
                .roles
                .default
                .as_deref()
                .map(|d| format!("★ {d}"))
                .unwrap_or("none".into())
        );
        buf.set_string(
            area.right().saturating_sub(width(&def) as u16 + 2),
            area.y,
            &def,
            t.faint().bg(bg),
        );
        let body = Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height.saturating_sub(1),
        );
        self.role_area = body;
        ctx.control(ROLES, body, false);
        let name_w = self
            .role_rows
            .iter()
            .map(|r| match r {
                RoleRow::Role(n) => width(n),
                RoleRow::Load => 0,
            })
            .max()
            .unwrap_or(8)
            .clamp(8, 24) as u16;
        for (i, row) in self.role_rows.iter().enumerate() {
            let y = body.y + i as u16;
            if y >= body.bottom() {
                break;
            }
            let rid = ROLES.child(i);
            let mut s = ctx.state(rid);
            s.focused = focused && i == self.role_cursor;
            s.selected = i == self.role_cursor;
            let st = t.row(s, bg);
            let rect = Rect::new(body.x, y, body.width, 1);
            fill(buf, rect, st);
            buf.set_string(rect.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            buf.set_string(
                rect.x + 1,
                y,
                if s.selected { "›" } else { " " },
                st.fg(if s.focused {
                    t.accent
                } else {
                    t.text_secondary
                }),
            );
            match row {
                RoleRow::Load => {
                    buf.set_string(
                        rect.x + 4,
                        y,
                        "+ Load role…",
                        st.fg(if s.focused {
                            t.text_primary
                        } else {
                            t.text_secondary
                        }),
                    );
                }
                RoleRow::Role(name) => {
                    let entry = w.roles.iter().find(|r| &r.name == name);
                    let allowed = self.pending.roles.allows(name);
                    let is_default = self.pending.roles.default.as_deref() == Some(name.as_str());
                    let changed = self.original.roles.allows(name) != allowed
                        || (self.original.roles.default.as_deref() == Some(name.as_str()))
                            != is_default;
                    buf.set_string(
                        rect.x + 3,
                        y,
                        if changed { "•" } else { " " },
                        st.fg(t.warning),
                    );
                    buf.set_string(
                        rect.x + 5,
                        y,
                        if allowed { "[✓]" } else { "[ ]" },
                        st.fg(if allowed {
                            t.text_primary
                        } else {
                            t.text_muted
                        }),
                    );
                    buf.set_string(
                        rect.x + 9,
                        y,
                        fit(name, name_w as usize),
                        if s.selected {
                            st.add_modifier(Modifier::BOLD)
                        } else {
                            st
                        },
                    );
                    let sx = rect.x + 10 + name_w;
                    buf.set_string(
                        sx,
                        y,
                        if is_default { "★" } else { " " },
                        st.fg(if s.focused {
                            t.text_primary
                        } else {
                            t.text_secondary
                        })
                        .add_modifier(if s.focused {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                    );
                    let (meta, tone) = match entry {
                        Some(RoleEntry {
                            load_error: Some(e),
                            ..
                        }) => (format!("! load error · {e}"), t.error),
                        Some(e) if !e.in_registry => {
                            (format!("{} · not in registry", e.source.label()), t.warning)
                        }
                        Some(e) => (
                            format!(
                                "registry · {} · {}",
                                if e.trusted { "trusted" } else { "untrusted" },
                                e.description
                            ),
                            t.text_muted,
                        ),
                        None => ("not in registry".into(), t.warning),
                    };
                    let mx = sx + 3;
                    buf.set_string(
                        mx,
                        y,
                        truncate(&meta, rect.right().saturating_sub(mx + 1) as usize),
                        st.fg(tone),
                    );
                }
            }
            ctx.clickable(rid, rect);
        }
    }

    fn roles_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        let n = self.role_rows.len();
        let Some(row) = self.role_rows.get(self.role_cursor).cloned() else {
            return Outcome::Ignored;
        };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.role_cursor = self.role_cursor.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.role_cursor = (self.role_cursor + 1).min(n - 1)
            }
            KeyCode::Home | KeyCode::Char('g') => self.role_cursor = 0,
            KeyCode::End | KeyCode::Char('G') => self.role_cursor = n - 1,
            KeyCode::Char(' ') => {
                if let RoleRow::Role(name) = &row {
                    let all: Vec<RoleName> = w.roles.iter().map(|r| r.name.clone()).collect();
                    let mut list = match &self.pending.roles.allowed {
                        AllowedRoles::All => all.clone(),
                        AllowedRoles::Custom(l) => l.clone(),
                    };
                    if list.iter().any(|r| r == name) {
                        list.retain(|r| r != name);
                        if self.pending.roles.default.as_deref() == Some(name.as_str()) {
                            self.pending.roles.default = None;
                        }
                    } else {
                        list.push(name.clone());
                    }
                    self.pending.roles.allowed =
                        if list.len() == all.len() && all.iter().all(|r| list.contains(r)) {
                            AllowedRoles::All
                        } else {
                            AllowedRoles::Custom(list)
                        };
                    cx.status(format!(
                        "{name} · {}",
                        if self.pending.roles.allows(name) {
                            "allowed"
                        } else {
                            "not allowed"
                        }
                    ));
                }
            }
            KeyCode::Enter | KeyCode::Char('*') => match &row {
                RoleRow::Load => self.open_role_picker(w, cx),
                RoleRow::Role(name) => {
                    if let Some(e) = w.roles.iter().find(|r| &r.name == name)
                        && e.load_error.is_some()
                    {
                        self.pending_load = Some(name.clone());
                        self.open_trust_dialog(e, cx);
                    } else if !self.pending.roles.allows(name) {
                        cx.error(format!(
                            "{name} must be allowed before it can be the default"
                        ));
                    } else if self.pending.roles.default.as_deref() == Some(name.as_str()) {
                        self.pending.roles.default = None;
                        cx.status(format!("{name} · no longer the default"));
                    } else {
                        self.pending.roles.default = Some(name.clone());
                        cx.status(format!("Default role ★ {name}"));
                    }
                }
            },
            KeyCode::Char('a') => self.open_role_picker(w, cx),
            _ => return Outcome::Ignored,
        }
        Outcome::Changed
    }

    fn open_role_picker(&mut self, w: &World, cx: &mut Cx) {
        let mut p = Picker::new(WidgetId::of("editor.roles.picker"), "Load role");
        p.width = 84;
        p.scope = Some("trusted role registry".into());
        let (items, targets) = self.role_items(w, "");
        p.set_items(items);
        self.picker_targets = targets;
        cx.open(Modal::Picker(p), ModalTag::new("role.load"));
    }

    fn role_items(&self, w: &World, query: &str) -> (Vec<PickerItem>, Vec<String>) {
        let q = query.to_lowercase();
        let mut items = vec![];
        let mut targets = vec![];
        for r in &w.roles {
            let label = r.full_name();
            if !q.is_empty()
                && !label.to_lowercase().contains(&q)
                && !r.description.to_lowercase().contains(&q)
            {
                continue;
            }
            let already = self.pending.roles.allows(&r.name)
                && matches!(self.pending.roles.allowed, AllowedRoles::Custom(_));
            items.push(PickerItem {
                label,
                detail: format!("{} · {}", r.source.label(), r.description),
                glyph: if r.trusted { "◆" } else { "!" },
                group: match &r.source {
                    RoleSource::Git { .. } => "git",
                    RoleSource::Local { .. } => "local",
                },
                tag: if !r.trusted {
                    Some("trust required")
                } else if already {
                    Some("allowed")
                } else {
                    None
                },
                matched: vec![],
                disabled: false,
            });
            targets.push(r.name.clone());
        }
        (items, targets)
    }

    fn open_trust_dialog(&mut self, e: &RoleEntry, cx: &mut Cx) {
        let repo = e.source.label();
        let facts = vec![
            Prop::new("Role", e.full_name()),
            Prop::new("Repository", repo),
        ];
        let d = Dialog::facts(
            WidgetId::of("editor.trust"),
            "Trust role source",
            facts,
            vec![
                "Trust this role source?".into(),
                "Dockerfile can run during image builds.".into(),
                "The role can access mounted workspace files.".into(),
            ],
            None,
            Button::primary(WidgetId::of("editor.trust.ok"), "Trust & load"),
        );
        cx.open(Modal::Dialog(d), ModalTag::new("role.trust").key(&e.name));
    }

    fn start_role_load(&mut self, name: &str, w: &mut World, cx: &mut Cx) {
        self.pending_load = Some(name.to_owned());
        let ok = name != "data-eng" || w.roles.iter().any(|r| r.name == name && r.trusted);
        w.schedule(
            1100,
            Msg::RoleLoaded {
                role: name.to_owned(),
                ok,
                error: if ok {
                    None
                } else {
                    Some(
                        "manifest fetch failed: 403 from github.com/acme-labs/roles-experimental"
                            .into(),
                    )
                },
            },
        );
        cx.status(format!("Loading role {name}…"));
    }

    fn finish_role_load(
        &mut self,
        name: &str,
        ok: bool,
        error: Option<String>,
        w: &mut World,
        cx: &mut Cx,
    ) {
        self.pending_load = None;
        match w.roles.iter_mut().find(|r| r.name == name) {
            Some(r) if ok => {
                r.load_error = None;
                r.in_registry = true;
                r.trusted = true;
                let all: Vec<RoleName> = w.roles.iter().map(|r| r.name.clone()).collect();
                let mut list = match &self.pending.roles.allowed {
                    AllowedRoles::All => all.clone(),
                    AllowedRoles::Custom(l) => l.clone(),
                };
                if !list.iter().any(|r| r == name) {
                    list.push(name.to_owned());
                }
                self.pending.roles.allowed = if list.len() == all.len() {
                    AllowedRoles::All
                } else {
                    AllowedRoles::Custom(list)
                };
                cx.status(format!("Role {name} loaded and allowed"));
            }
            Some(r) => {
                r.load_error = error.clone();
                let d = InfoDialog::new(
                    WidgetId::of("editor.roleerr"),
                    &format!("Role {name} failed to load"),
                    vec![
                        Prop::new("Error", error.unwrap_or("unknown".into())),
                        Prop::new("Source", r.source.label()),
                        Prop::new("Next", "trust stays granted · retry from the Roles tab"),
                    ],
                )
                .error()
                .width(70);
                cx.open(Modal::Info(d), ModalTag::new("roleerr"));
            }
            None => cx.error(format!("Role {name} is not in the registry")),
        }
        self.build_roles(w);
    }

    fn open_workdir_picker(&mut self, w: &World, cx: &mut Cx) {
        let mut p = Picker::new(WidgetId::of("editor.workdir.picker"), "Working directory");
        p.width = 72;
        p.scope = Some(self.pending.name.clone());
        let mut items = vec![];
        let mut targets = vec![];
        for m in &self.cfg.pending.mounts {
            items.push(PickerItem {
                label: m.destination.clone(),
                detail: format!("mount · {}", w.tilde(m.source_label())),
                glyph: "▪",
                group: "mounts",
                tag: if m.destination == self.pending.workdir {
                    Some("current")
                } else {
                    None
                },
                matched: vec![],
                disabled: false,
            });
            targets.push(m.destination.clone());
            if let crate::domain::workspace::MountSource::Host(src) = &m.source {
                let src = crate::sim::world::expand(&w.home, src);
                for f in w.fs.iter().filter(|f| {
                    f.dir
                        && f.path.starts_with(&format!("{src}/"))
                        && !f.path[src.len() + 1..].contains('/')
                }) {
                    let rel = &f.path[src.len()..];
                    let dest = format!("{}{rel}", m.destination);
                    items.push(PickerItem {
                        label: dest.clone(),
                        detail: f.meta.clone(),
                        glyph: "·",
                        group: "subdirectories",
                        tag: if dest == self.pending.workdir {
                            Some("current")
                        } else {
                            None
                        },
                        matched: vec![],
                        disabled: false,
                    });
                    targets.push(dest);
                }
            }
        }
        if items.is_empty() {
            items.push(PickerItem {
                label: self.pending.workdir.clone(),
                detail: "no mounts yet · add one in the Mounts tab".into(),
                glyph: "▪",
                group: "",
                tag: Some("current"),
                matched: vec![],
                disabled: false,
            });
            targets.push(self.pending.workdir.clone());
        }
        p.set_items(items);
        self.picker_targets = targets;
        cx.open(Modal::Picker(p), ModalTag::new("workdir"));
    }

    fn general_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        let focus = cx.focus.current();
        if focus == Some(NAME) {
            let (o, ev) = self.name.on_key(key);
            match ev {
                Some(InputEvent::Committed) => {
                    self.name.error = None;
                    self.name.validate();
                    self.sync_pending();
                    cx.status(format!("Name {} · save to apply", self.pending.name));
                    return Outcome::Changed;
                }
                Some(InputEvent::Cancelled) => {
                    cx.status("Reverted");
                    return Outcome::Changed;
                }
                _ => {}
            }
            if o.consumed() {
                return o;
            }
        }
        if focus == Some(WORKDIR) {
            let (o, fired) = self.workdir_button.on_key(key);
            if fired {
                self.open_workdir_picker(w, cx);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if focus == Some(KEEP_AWAKE) {
            let o = self.keep_awake.on_key(key);
            if o.consumed() {
                self.sync_pending();
                return o;
            }
        }
        if focus == Some(GIT_PULL) {
            let o = self.git_pull.on_key(key);
            if o.consumed() {
                self.sync_pending();
                return o;
            }
        }
        if focus == Some(DIRTY_POLICY) {
            let (o, _) = self.dirty_policy.on_key(key);
            if o.consumed() {
                self.sync_pending();
                return o;
            }
        }
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if !self.name.editing => {
                cx.focus.next(cx.ring);
                Outcome::Changed
            }
            KeyCode::Up | KeyCode::Char('k') if !self.name.editing => {
                cx.focus.prev(cx.ring);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }
}

impl EditorScreen {
    /// Names of other workspaces (filled on enter) for uniqueness checks.
    fn refresh_taken(&mut self, w: &World) {
        self.taken = w
            .workspaces
            .iter()
            .filter(|x| Some(x.id) != self.workspace)
            .map(|x| (x.name.clone(), x.id))
            .collect();
    }
}

impl Screen for EditorScreen {
    fn enter(&mut self, w: &mut World, cx: &mut Cx) {
        self.refresh_taken(w);
        self.build_roles(w);
        cx.focus.focus(if self.is_create() { NAME } else { TABS });
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(TABS)
    }

    fn is_editing(&self) -> bool {
        self.name.editing
    }

    fn animating(&self, _w: &World) -> bool {
        self.saving || self.pending_load.is_some()
    }

    fn on_tick(&mut self, _w: &mut World, _cx: &mut Cx) -> Outcome {
        if self.saving || self.pending_load.is_some() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    fn on_msg(&mut self, msg: &Msg, w: &mut World, cx: &mut Cx) -> Outcome {
        match msg {
            Msg::WorkspaceSaved { ok, .. } if self.saving => {
                self.finish_save(*ok, w, cx);
                Outcome::Changed
            }
            Msg::RoleLoaded { role, ok, error }
                if self.pending_load.as_deref() == Some(role.as_str()) =>
            {
                self.finish_role_load(role, *ok, error.clone(), w, cx);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.saving {
            return Outcome::Consumed;
        }
        let focus = cx.focus.current();
        let editing = self.is_editing();
        // global chords
        if key.ctrl() && key.code == KeyCode::Char('s') {
            self.open_preview(w, cx);
            return Outcome::Changed;
        }
        if !editing {
            match key.code {
                KeyCode::Char('[') => {
                    let i = (self.tabs.active + 4) % 5;
                    self.set_tab(i, cx);
                    return Outcome::Changed;
                }
                KeyCode::Char(']') => {
                    let i = (self.tabs.active + 1) % 5;
                    self.set_tab(i, cx);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if focus == Some(TABS) {
            if matches!(
                key.code,
                KeyCode::Enter | KeyCode::Down | KeyCode::Char('j')
            ) {
                cx.focus.focus(self.body_focus());
                return Outcome::Changed;
            }
            let (o, ev) = self.tabs.on_key(key);
            if let Some(TabEvent::Activated(i)) = ev {
                self.tab = EdTab::ALL[i.min(4)];
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
            match key.code {
                KeyCode::Esc => {
                    self.leave(cx);
                    return Outcome::Changed;
                }
                _ => return Outcome::Ignored,
            }
        }
        if focus == Some(CANCEL) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                self.leave(cx);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if focus == Some(SAVE) {
            let (o, fired) = self.save.on_key(key);
            if fired {
                self.open_preview(w, cx);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if matches!(focus, Some(CANCEL) | Some(SAVE)) {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    cx.focus.focus(CANCEL);
                    return Outcome::Changed;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    cx.focus.focus(SAVE);
                    return Outcome::Changed;
                }
                KeyCode::Esc | KeyCode::Up | KeyCode::Char('k') => {
                    cx.focus.focus(self.body_focus());
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        // body
        let body = match self.tab {
            EdTab::General
                if matches!(
                    focus,
                    Some(NAME)
                        | Some(WORKDIR)
                        | Some(KEEP_AWAKE)
                        | Some(GIT_PULL)
                        | Some(DIRTY_POLICY)
                ) =>
            {
                self.general_key(key, w, cx)
            }
            EdTab::Roles if focus == Some(ROLES) => self.roles_key(key, w, cx),
            t if t.cfg().is_some() && focus == Some(self.cfg.list_id(t.cfg().unwrap())) => {
                let o = self.cfg.on_key(t.cfg().unwrap(), key, w, cx);
                if o.consumed() {
                    self.sync_pending();
                }
                o
            }
            _ => Outcome::Ignored,
        };
        if body.consumed() {
            return body;
        }
        if editing {
            return Outcome::Ignored;
        }
        match key.code {
            KeyCode::Esc => {
                if self.name.error.is_some() {
                    self.name.error = None;
                }
                cx.focus.focus(TABS);
                Outcome::Changed
            }
            KeyCode::Char('c') if key.plain() && self.tab == EdTab::Auth => {
                cx.go(Go::Accounts { select: None });
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.saving {
            return Outcome::Consumed;
        }
        if self.tabs.owns(id) {
            let (o, ev) = self.tabs.on_click(id);
            if let Some(TabEvent::Activated(i)) = ev {
                self.tab = EdTab::ALL[i.min(4)];
            }
            cx.focus.focus(TABS);
            return o.or(Outcome::Changed);
        }
        if id == CANCEL {
            cx.focus.focus(CANCEL);
            if self.cancel.on_click() {
                self.leave(cx);
            }
            return Outcome::Changed;
        }
        if id == SAVE {
            cx.focus.focus(SAVE);
            if self.save.on_click() {
                self.open_preview(w, cx);
            }
            return Outcome::Changed;
        }
        match self.tab {
            EdTab::General => {
                if id == NAME {
                    let was = cx.focus.is(NAME);
                    cx.focus.focus(NAME);
                    return self.name.on_click(pos, was);
                }
                if id == WORKDIR {
                    cx.focus.focus(WORKDIR);
                    if self.workdir_button.on_click() {
                        self.open_workdir_picker(w, cx);
                    }
                    return Outcome::Changed;
                }
                if id == KEEP_AWAKE {
                    cx.focus.focus(KEEP_AWAKE);
                    let o = self.keep_awake.on_click();
                    self.sync_pending();
                    return o;
                }
                if id == GIT_PULL {
                    cx.focus.focus(GIT_PULL);
                    let o = self.git_pull.on_click();
                    self.sync_pending();
                    return o;
                }
                if self.dirty_policy.owns(id) {
                    cx.focus.focus(DIRTY_POLICY);
                    let (o, _) = self.dirty_policy.on_click(id);
                    self.sync_pending();
                    return o;
                }
                Outcome::Ignored
            }
            EdTab::Roles => {
                for i in 0..self.role_rows.len() {
                    if ROLES.child(i) == id {
                        let same = self.role_cursor == i;
                        self.role_cursor = i;
                        cx.focus.focus(ROLES);
                        if same {
                            let k = Key {
                                code: KeyCode::Char(' '),
                                mods: ratatui::crossterm::event::KeyModifiers::NONE,
                            };
                            return self.roles_key(&k, w, cx);
                        }
                        return Outcome::Changed;
                    }
                }
                Outcome::Ignored
            }
            t => {
                let o = self.cfg.on_click(t.cfg().unwrap(), id, pos, w, cx);
                if o.consumed() {
                    self.sync_pending();
                }
                o
            }
        }
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        match self.tab.cfg() {
            Some(t) => self.cfg.on_wheel(t, id, delta),
            None => Outcome::Ignored,
        }
    }

    fn on_paste(&mut self, text: &str, _w: &mut World) -> Outcome {
        if self.name.editing {
            self.name.on_paste(text)
        } else {
            Outcome::Ignored
        }
    }

    fn picker_items(&mut self, tag: &ModalTag, query: &str, w: &World) -> Option<Vec<PickerItem>> {
        if tag.kind == "role.load" {
            let (items, targets) = self.role_items(w, query);
            self.picker_targets = targets;
            return Some(items);
        }
        None
    }

    fn form_changed(&mut self, tag: &ModalTag, form: &mut super::modals::FormDialog, w: &World) {
        self.cfg.form_changed(tag, form, w);
    }

    fn on_modal(
        &mut self,
        tag: &ModalTag,
        result: ModalResult,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        if let Some(o) = self.cfg.on_modal(tag, result.clone(), w, cx) {
            self.sync_pending();
            return o;
        }
        match (tag.kind, result) {
            (
                "preview",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                self.start_save(w, cx);
                Outcome::Changed
            }
            (
                "preview.cleanup",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                self.cfg.clean_up();
                cx.status("Cleaned up isolated state · saving");
                self.start_save(w, cx);
                Outcome::Changed
            }
            ("preview" | "preview.cleanup", _) => {
                cx.status("Not saved · keep editing");
                cx.focus.focus(SAVE);
                Outcome::Changed
            }
            (
                "exit",
                ModalResult::Dialog {
                    action: Some(2), ..
                },
            ) => {
                self.open_preview(w, cx);
                Outcome::Changed
            }
            (
                "exit",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                cx.status(format!(
                    "Discarded {}",
                    plural(self.change_count().max(1), "change", "changes")
                ));
                cx.go(Go::Manager);
                Outcome::Changed
            }
            ("exit", _) => Outcome::Changed,
            ("savefail", _) => {
                cx.focus.focus(SAVE);
                Outcome::Changed
            }
            ("workdir", ModalResult::Picked(i)) => {
                if let Some(d) = self.picker_targets.get(i).cloned() {
                    self.pending.workdir = d.clone();
                    cx.status(format!("Working directory {d} · save to apply"));
                }
                cx.focus.focus(WORKDIR);
                Outcome::Changed
            }
            ("role.load", ModalResult::Picked(i)) => {
                let Some(name) = self.picker_targets.get(i).cloned() else {
                    return Outcome::Changed;
                };
                let entry = w.roles.iter().find(|r| r.name == name).cloned();
                match entry {
                    Some(e) if !e.trusted => {
                        self.pending_load = Some(name);
                        self.open_trust_dialog(&e, cx);
                    }
                    Some(_) => self.start_role_load(&name, w, cx),
                    None => cx.error(format!("Role {name} not found")),
                }
                cx.focus.focus(ROLES);
                Outcome::Changed
            }
            (
                "role.trust",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                let name = tag.key.clone();
                if let Some(r) = w.roles.iter_mut().find(|r| r.name == name) {
                    r.trusted = true;
                }
                if let Some(t) = w
                    .global
                    .trust
                    .iter_mut()
                    .find(|t| t.source.contains("roles-experimental"))
                {
                    t.trusted = true;
                }
                self.start_role_load(&name, w, cx);
                Outcome::Changed
            }
            ("role.trust", _) => {
                self.pending_load = None;
                cx.status("Not trusted · role not loaded");
                Outcome::Changed
            }
            ("roleerr", _) => Outcome::Changed,
            _ => Outcome::Changed,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        self.sync_pending();
        let items: Vec<TabItem> = EdTab::ALL
            .iter()
            .zip(TAB_NAMES)
            .map(|(tab, name)| {
                let mut it = TabItem::new(name);
                it.dirty = match tab {
                    EdTab::General => self.general_dirty(),
                    EdTab::Roles => self.roles_dirty(),
                    t => self.cfg.tab_dirty(t.cfg().unwrap()),
                };
                it.error = match tab {
                    EdTab::General => self.name.error.is_some(),
                    EdTab::Roles => false,
                    t => self.cfg.tab_error(t.cfg().unwrap()),
                };
                it
            })
            .collect();
        let active = self.tabs.active;
        self.tabs = Tabs::with_items(TABS, items);
        self.tabs.set_active(active);
        self.tabs.quiet = false;
        self.tabs.render(
            Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), 2),
            buf,
            ctx,
            bg,
        );
        let body = Rect::new(
            area.x + 1,
            area.y + 4,
            area.width.saturating_sub(2),
            area.height.saturating_sub(6),
        );
        self.row_status = None;
        match self.tab {
            EdTab::General => self.render_general(body, buf, ctx, w),
            EdTab::Roles => self.render_roles(body, buf, ctx, w),
            t => {
                self.row_status = self.cfg.render(t.cfg().unwrap(), body, buf, ctx, w);
            }
        }
        // action row
        let n = self.change_count();
        self.save.disabled = n == 0 && !self.is_create() && !self.saving;
        self.save.label = if self.saving {
            format!("{} Saving…", spinner_frame(w.now_ms() as u64 / 80))
        } else {
            "Save…".into()
        };
        let widths = [self.cancel.width(), self.save.width()];
        let rects = row_layout_right(
            Rect::new(
                area.x,
                area.bottom().saturating_sub(1),
                area.width.saturating_sub(4),
                1,
            ),
            &widths,
            3,
        );
        self.cancel.render(rects[0], buf, ctx, bg);
        self.save.render(rects[1], buf, ctx, bg);
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if self.saving {
            return vec![hint("", "Saving…")];
        }
        let mut v = vec![];
        if focus == Some(TABS) {
            v.push(hint("← →", "Tab"));
            v.push(hint("1–5", "Jump"));
            v.push(hint("Enter", "Body"));
        } else if matches!(focus, Some(CANCEL) | Some(SAVE)) {
            v.push(hint("← →", "Choose"));
            v.push(hint("Enter", "Run"));
        } else {
            match self.tab {
                EdTab::General => {
                    if focus == Some(NAME) {
                        v.push(if self.name.editing {
                            hint("Enter", "Commit")
                        } else {
                            hint("Enter", "Edit")
                        });
                    } else if focus == Some(WORKDIR) {
                        v.push(hint("Enter", "Choose…"));
                    } else {
                        v.push(hint("Space", "Toggle"));
                    }
                    v.push(hint("Tab", "Next"));
                }
                EdTab::Roles => match self.role_rows.get(self.role_cursor) {
                    Some(RoleRow::Load) => v.push(hint("Enter", "Load role…")),
                    _ => {
                        v.push(hint("Space", "Allow"));
                        v.push(hint("Enter", "Set default"));
                        v.push(hint("a", "Load role…"));
                    }
                },
                t => v.extend(self.cfg.hints(t.cfg().unwrap())),
            }
        }
        v.push(hint("[ ]", "Switch tab"));
        v.push(hint("Ctrl+S", "Save"));
        v.push(hint("Esc", "Back"));
        v
    }

    fn crumb(&self, _w: &World) -> String {
        if self.is_create() {
            "Workspaces › new workspace › edit".into()
        } else {
            format!("Workspaces › {} › edit", self.original.name)
        }
    }

    fn strip_right(&self, w: &World) -> Vec<Segment> {
        let mut v = vec![];
        if self.saving {
            v.push(
                Segment::new(
                    format!("{} saving…", spinner_frame(w.now_ms() as u64 / 80)),
                    Tone::Secondary,
                )
                .priority(9),
            );
        }
        let n = self.change_count();
        if n > 0 || self.is_create() {
            v.push(
                Segment::new(
                    format!(
                        "• {}{}",
                        plural(n.max(1), "change", "changes"),
                        if self.is_create() { " · new" } else { "" }
                    ),
                    Tone::Warning,
                )
                .priority(8),
            );
        }
        if let Some(s) = &self.row_status {
            v.push(Segment::new(s.clone(), Tone::Muted).priority(3));
        }
        v
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        if cx.focus.is(TABS) {
            self.leave(cx);
        } else {
            cx.focus.focus(TABS);
        }
        Outcome::Changed
    }
}
