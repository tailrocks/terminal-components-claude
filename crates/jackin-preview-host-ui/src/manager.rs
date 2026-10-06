//! Manager route state and registered control ids.
//!
//! The manager owns selection semantics while `tui-next::List` owns focus and
//! painting.  Keeping the ids here gives the route and its tests one stable
//! vocabulary without copying a generic widget implementation into the app.

use ratatui::layout::Rect;
use termrock::author::{
    Family, FgStep, Focusability, Id, ItemKey, Modifier, PaintStyle, Part, Role, StateFlags, StyleDefaults,
    StylePatch, Surface, Ui, Variant,
};
use termrock::{
    Button, Empty, EmptyState, Insets, LayerSize, LayerSpec, ListState, Panel, PanelKind, Props,
    PropsRow, ScrollState, SplitAxis, SplitPane, SplitPaneState, truncate, truncate_middle, width,
};

use crate::manager_actions::{role_label, Fact};
use jackin_preview_domain::account::AccountId;
use jackin_preview_domain::agent::Agent;
use jackin_preview_domain::clock::format_duration;
use jackin_preview_domain::instance::{DaemonSnapshot, InstanceStatus};
use jackin_preview_domain::workspace::{AllowedRoles, EnvValue, WorkspaceId};
use jackin_preview_sim::world::{DaemonHealth, World};

const FAINT_DETAIL_PATCH: [(Part, StylePatch); 1] = [
    (Part::DETAIL, StylePatch::new().set_fg(Role::Fg(FgStep::Faint))),
];

/// Manager tree control.
pub const TREE: Id = Id::root("jackin.manager.tree");
/// New-workspace action.
pub const NEW_WORKSPACE: Id = Id::root("jackin.manager.new-workspace");
/// Instance detail panel.
pub const DETAIL: Id = Id::root("jackin.manager.detail");
/// Launch action owned by the manager.
pub const LAUNCH: Id = Id::root("jackin.manager.launch");
/// Agent picker opened by the launch action.
pub const AGENT_PICKER: Id = LAUNCH.sub("agent-picker");
/// Roster panel for wide layout.
pub const ROSTER: Id = Id::root("jackin.manager.roster");
/// Split seam between panes.
pub const SEAM: Id = Id::root("jackin.manager.seam");
/// Inspect dialog owned by the manager.
pub const INSPECT: Id = Id::root("jackin.manager.inspect");
/// Close button in the inspect dialog.
pub const INSPECT_CLOSE: Id = INSPECT.sub("close");

/// Stable identity for a row in the manager tree.
///
/// The visible row text is deliberately not used as identity: workspace and
/// instance labels can change while a cursor is still pointing at the same
/// durable object.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ManagerRowKey {
    /// The unsaved current directory row.
    #[default]
    CurrentDirectory,
    /// A saved workspace row.
    Workspace(WorkspaceId),
    /// A persisted instance child row.
    Instance(String),
    /// The create-workspace action row.
    NewWorkspace,
}

impl ManagerRowKey {
    /// Return a stable, non-display key suitable for a keyed list item.
    pub fn stable_key(&self) -> String {
        match self {
            Self::CurrentDirectory => "current-directory".into(),
            Self::Workspace(id) => format!("workspace:{id}"),
            Self::Instance(id) => format!("instance:{id}"),
            Self::NewWorkspace => "new-workspace".into(),
        }
    }

    /// Return the workspace represented by this row, if any.
    pub const fn workspace(&self) -> Option<WorkspaceId> {
        match self {
            Self::Workspace(id) => Some(*id),
            Self::CurrentDirectory | Self::Instance(_) | Self::NewWorkspace => None,
        }
    }
}

/// One agent offered by the manager's launch picker.
///
/// Candidates come from [`World::offered_agents`].  That source filters out
/// agents with no configured account; a configured but unusable account stays
/// represented as a blocked candidate so the operator can see why launch is
/// unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCandidate {
    /// Runtime selected by the candidate.
    pub agent: Agent,
    /// Preselected ready account, when one exists.
    pub account: Option<AccountId>,
    /// Human-readable reason when the configured account is unusable.
    pub blocked: Option<String>,
}

impl LaunchCandidate {
    /// Whether this candidate can start a session immediately.
    pub const fn is_ready(&self) -> bool {
        self.account.is_some() && self.blocked.is_none()
    }
}

/// State owned by the manager route.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManagerState {
    /// Public-list selection state owned by this route.
    pub list: ListState,
    expanded: Vec<WorkspaceId>,
    /// Monotonic key for derived row projections.
    rows_revision: u64,
    selected_workspace: Option<WorkspaceId>,
    selected_row: ManagerRowKey,
    detail_open: bool,
}

impl ManagerState {
    /// Whether a workspace tree node is expanded.
    pub fn is_expanded(&self, id: WorkspaceId) -> bool {
        self.expanded.contains(&id)
    }

    /// Toggle one workspace node.
    pub fn toggle(&mut self, id: WorkspaceId) {
        if let Some(index) = self.expanded.iter().position(|value| *value == id) {
            self.expanded.remove(index);
        } else {
            self.expanded.push(id);
        }
        self.rows_revision = self.rows_revision.wrapping_add(1);
    }

    /// Invalidate the derived row projection after a world refresh.
    pub fn invalidate_rows(&mut self) {
        self.rows_revision = self.rows_revision.wrapping_add(1);
    }

    /// Revision of the expanded-row projection.
    pub const fn rows_revision(&self) -> u64 {
        self.rows_revision
    }

    /// Select a workspace for detail inspection.
    pub fn select(&mut self, id: Option<WorkspaceId>) {
        self.select_row(match id {
            Some(id) => ManagerRowKey::Workspace(id),
            None => ManagerRowKey::CurrentDirectory,
        });
    }

    /// Selected workspace, if any.
    pub const fn selected(&self) -> Option<WorkspaceId> {
        self.selected_workspace
    }

    /// Current stable tree-row selection.
    pub const fn selected_row(&self) -> &ManagerRowKey {
        &self.selected_row
    }

    /// Select a tree row while keeping the workspace projection in sync.
    pub fn select_row(&mut self, row: ManagerRowKey) {
        self.selected_workspace = row.workspace();
        self.selected_row = row;
    }

    /// Build launch candidates for the selected workspace scope.
    ///
    /// This is the one manager-owned path into launch-agent discovery. It
    /// preserves the world's configured/unconfigured distinction instead of
    /// rebuilding a positional list from all supported [`Agent`] values.
    pub fn launch_candidates(
        world: &World,
        workspace: Option<WorkspaceId>,
        role: Option<&str>,
    ) -> Vec<LaunchCandidate> {
        let workspace_ref = workspace.and_then(|id| world.workspace(id));
        world
            .offered_agents(workspace_ref, role)
            .into_iter()
            .map(|(agent, offer)| LaunchCandidate {
                agent,
                account: offer.preselected,
                blocked: offer.blocked,
            })
            .collect()
    }

    /// Open or close the detail projection.
    pub const fn set_detail_open(&mut self, open: bool) {
        self.detail_open = open;
    }

    /// Whether the detail projection is visible.
    pub const fn detail_open(&self) -> bool {
        self.detail_open
    }
}

fn resolve_style(ui: &Ui<'_>, fg: Role, bg: Role, bold: bool) -> PaintStyle {
    let mut patch = StylePatch::new().set_fg(fg).set_bg(bg);
    if bold {
        patch = patch.add(Modifier::BOLD);
    } else {
        patch = patch.remove(Modifier::BOLD);
    }
    ui.style_defaults(
        Family::PANEL,
        Variant::DEFAULT,
        Part::custom("jackin.manager.style"),
        StateFlags::empty(),
        StyleDefaults::new(patch),
        None,
    )
    .style
}

pub struct ManagerPalette {
    pub canvas: PaintStyle,
    pub gutter_unfocused: PaintStyle,
    pub border_on_canvas: PaintStyle,
    pub primary_on_canvas: PaintStyle,
    pub primary_on_canvas_bold: PaintStyle,
    pub secondary_on_canvas: PaintStyle,
    pub muted_on_canvas: PaintStyle,
    pub faint_on_canvas: PaintStyle,
    pub accent_on_canvas: PaintStyle,
    pub danger_on_canvas: PaintStyle,
    pub warning_on_canvas: PaintStyle,
    pub primary_on_accent_tint_bold: PaintStyle,
    pub accent_on_accent_tint_bold: PaintStyle,
    pub secondary_on_accent_tint: PaintStyle,
    pub muted_on_accent_tint: PaintStyle,
    pub card_bg: PaintStyle,
    pub card_border: PaintStyle,
    pub card_primary: PaintStyle,
    pub card_primary_bold: PaintStyle,
    pub card_secondary: PaintStyle,
    pub card_secondary_bold: PaintStyle,
    pub card_muted: PaintStyle,
    pub card_faint: PaintStyle,
    pub card_danger: PaintStyle,
    pub card_warning: PaintStyle,
}

impl ManagerPalette {
    pub fn new(ui: &Ui<'_>) -> Self {
        let canvas = Role::Surface(Surface::Canvas);
        let surface = Role::Surface(Surface::Surface);
        Self {
            canvas: resolve_style(ui, Role::Fg(FgStep::Primary), canvas, false),
            gutter_unfocused: resolve_style(ui, canvas, canvas, false),
            border_on_canvas: resolve_style(ui, Role::BorderSubtle, canvas, false),
            primary_on_canvas: resolve_style(ui, Role::Fg(FgStep::Primary), canvas, false),
            primary_on_canvas_bold: resolve_style(ui, Role::Fg(FgStep::Primary), canvas, true),
            secondary_on_canvas: resolve_style(ui, Role::Fg(FgStep::Secondary), canvas, false),
            muted_on_canvas: resolve_style(ui, Role::Fg(FgStep::Muted), canvas, false),
            faint_on_canvas: resolve_style(ui, Role::Fg(FgStep::Faint), canvas, false),
            accent_on_canvas: resolve_style(ui, Role::Accent, canvas, false),
            danger_on_canvas: resolve_style(ui, Role::Danger, canvas, false),
            warning_on_canvas: resolve_style(ui, Role::Warning, canvas, false),
            primary_on_accent_tint_bold: resolve_style(ui, Role::Fg(FgStep::Primary), Role::AccentTint, true),
            accent_on_accent_tint_bold: resolve_style(ui, Role::Accent, Role::AccentTint, true),
            secondary_on_accent_tint: resolve_style(ui, Role::Fg(FgStep::Secondary), Role::AccentTint, false),
            muted_on_accent_tint: resolve_style(ui, Role::Fg(FgStep::Muted), Role::AccentTint, false),
            card_bg: resolve_style(ui, Role::Fg(FgStep::Primary), surface, false),
            card_border: resolve_style(ui, Role::BorderSubtle, surface, false),
            card_primary: resolve_style(ui, Role::Fg(FgStep::Primary), surface, false),
            card_primary_bold: resolve_style(ui, Role::Fg(FgStep::Primary), surface, true),
            card_secondary: resolve_style(ui, Role::Fg(FgStep::Secondary), surface, false),
            card_secondary_bold: resolve_style(ui, Role::Fg(FgStep::Secondary), surface, true),
            card_muted: resolve_style(ui, Role::Fg(FgStep::Muted), surface, false),
            card_faint: resolve_style(ui, Role::Fg(FgStep::Faint), surface, false),
            card_danger: resolve_style(ui, Role::Danger, surface, false),
            card_warning: resolve_style(ui, Role::Warning, surface, false),
        }
    }
}

pub fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}

pub fn position_label(scroll: &ScrollState) -> String {
    if !scroll.overflows() {
        return String::new();
    }
    let r = scroll.visible_range();
    format!("{}–{} of {}", r.start + 1, r.end, scroll.content_len())
}

fn fit(s: &str, w: u16) -> String {
    let t = truncate(s, w);
    let pad = w.saturating_sub(width(&t));
    format!("{t}{}", " ".repeat(pad as usize))
}

#[derive(Debug, Clone)]
pub struct ManagerRow {
    pub key: ManagerRowKey,
    pub depth: u16,
    pub glyph: &'static str,
    pub glyph_tone: Role,
    pub label: String,
    pub meta: String,
    pub meta_tone: Role,
    pub trailing: Option<(&'static str, Role)>,
    pub expandable: bool,
}

pub fn build_rows(world: &World, state: &ManagerState) -> Vec<ManagerRow> {
    let mut rows = Vec::new();
    let cwd_ws = world.cwd_workspace().map(|x| x.id);
    rows.push(ManagerRow {
        key: ManagerRowKey::CurrentDirectory,
        depth: 0,
        glyph: " ",
        glyph_tone: Role::Fg(FgStep::Primary),
        label: "Current directory".into(),
        meta: match cwd_ws.and_then(|id| world.workspace(id)) {
            Some(ws) => format!("saved as {}", ws.name),
            None => world.tilde(&world.cwd),
        },
        meta_tone: Role::Fg(FgStep::Muted),
        trailing: None,
        expandable: false,
    });
    for ws in &world.workspaces {
        let kids = world.instances_of(Some(ws.id));
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
        let (meta, tone) = if running > 0 {
            (format!("{running} running"), Role::Fg(FgStep::Secondary))
        } else if failed {
            ("! failed".into(), Role::Danger)
        } else if restore {
            ("restore available".into(), Role::Fg(FgStep::Secondary))
        } else if kids.is_empty() {
            ("idle".into(), Role::Fg(FgStep::Faint))
        } else {
            (plural(kids.len(), "record", "records"), Role::Fg(FgStep::Muted))
        };
        let expanded = state.is_expanded(ws.id);
        rows.push(ManagerRow {
            key: ManagerRowKey::Workspace(ws.id),
            depth: 0,
            glyph: if kids.is_empty() {
                " "
            } else if expanded {
                "▾"
            } else {
                "▸"
            },
            glyph_tone: Role::Fg(FgStep::Secondary),
            label: ws.name.clone(),
            meta,
            meta_tone: tone,
            trailing: None,
            expandable: !kids.is_empty(),
        });
        if expanded {
            for i in kids {
                let (glyph, gt) = match i.status {
                    InstanceStatus::Running => ("◉", Role::Fg(FgStep::Primary)),
                    InstanceStatus::Crashed | InstanceStatus::FailedSetup => ("!", Role::Danger),
                    _ => ("◌", Role::Fg(FgStep::Muted)),
                };
                let (meta, mt) = match i.status {
                    InstanceStatus::Running => (
                        format!(
                            "running {}",
                            format_duration((world.now_secs() - i.created_secs).max(0) as u64)
                        ),
                        Role::Fg(FgStep::Secondary),
                    ),
                    InstanceStatus::CleanExited => (
                        format!("exited clean · {}", world.clock.ago(i.last_seen_secs)),
                        Role::Fg(FgStep::Muted),
                    ),
                    InstanceStatus::Crashed => ("crashed · exit 137".into(), Role::Danger),
                    InstanceStatus::PreservedDirty => {
                        ("preserved · dirty".into(), Role::Fg(FgStep::Secondary))
                    }
                    InstanceStatus::PreservedUnpushed => {
                        ("preserved · unpushed".into(), Role::Fg(FgStep::Secondary))
                    }
                    InstanceStatus::RestoreAvailable => {
                        ("restore available".into(), Role::Fg(FgStep::Secondary))
                    }
                    InstanceStatus::FailedSetup => ("failed setup".into(), Role::Danger),
                    _ => (i.status.label().into(), Role::Fg(FgStep::Muted)),
                };
                rows.push(ManagerRow {
                    key: ManagerRowKey::Instance(i.id.clone()),
                    depth: 1,
                    glyph,
                    glyph_tone: gt,
                    label: format!(
                        "{}  {} · {}",
                        i.id.trim_start_matches("jk-"),
                        role_label(world, &i.role),
                        i.agent.label()
                    ),
                    meta,
                    meta_tone: mt,
                    trailing: if i.status.dirty() {
                        Some(("•", Role::Warning))
                    } else {
                        None
                    },
                    expandable: false,
                });
            }
        }
    }
    rows.push(ManagerRow {
        key: ManagerRowKey::NewWorkspace,
        depth: 0,
        glyph: " ",
        glyph_tone: Role::Fg(FgStep::Primary),
        label: "+ New workspace".into(),
        meta: String::new(),
        meta_tone: Role::Fg(FgStep::Muted),
        trailing: None,
        expandable: false,
    });
    rows
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetailRow {
    Pane { tab: usize, pane: u64, text: String },
    Text(String, Role),
    Session(String),
    Blank,
}

pub fn build_detail(world: &World, instance_id: &str) -> Vec<DetailRow> {
    let mut rows = Vec::new();
    if let Some(i) = world.instance(instance_id) {
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
                        Role::Fg(FgStep::Secondary),
                    ));
                    if t.active || tabs.len() <= 3 {
                        for (pi, p) in t.panes.iter().enumerate() {
                            let pane_id = world
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
                Role::Fg(FgStep::Muted),
            )),
            DaemonSnapshot::Unavailable => rows.push(DetailRow::Text(
                if i.status.is_live() {
                    "Daemon unavailable — showing manifest sessions".into()
                } else {
                    "No live daemon (instance not running)".into()
                },
                Role::Fg(FgStep::Muted),
            )),
        }
        rows.push(DetailRow::Blank);
        match &i.sessions {
            Ok(s) if s.is_empty() => {
                rows.push(DetailRow::Text("No sessions recorded".into(), Role::Fg(FgStep::Muted)))
            }
            Ok(s) => {
                for r in s {
                    rows.push(DetailRow::Session(format!(
                        "{:<5} {:<11} {:<8} {}",
                        r.id,
                        world.clock.ago(r.started_secs),
                        r.agent.map(|a| a.short()).unwrap_or("shell"),
                        r.status.label()
                    )));
                }
            }
            Err(e) => rows.push(DetailRow::Text(format!("! {}", e.label()), Role::Danger)),
        }
    }
    rows
}

#[derive(Debug, Clone)]
pub struct ActionButton {
    pub id: Id,
    pub label: &'static str,
    pub danger: bool,
}

pub fn rebuild_actions(world: &World, selected: &ManagerRowKey) -> Vec<ActionButton> {
    let mut v = Vec::new();
    let mk = |name: &'static str, label: &'static str, danger: bool| ActionButton {
        id: DETAIL.sub(name),
        label,
        danger,
    };
    match selected {
        ManagerRowKey::Instance(id) => {
            let Some(i) = world.instance(id) else {
                return v;
            };
            if i.status.is_live() {
                v.push(mk("reconnect", "Reconnect", false));
                v.push(mk("session", "New session", false));
                v.push(mk("shell", "Shell", false));
            } else if i.status.reconnectable() && i.status != InstanceStatus::Crashed {
                v.push(mk("reconnect", "Restore", false));
            }
            v.push(mk("inspect", "Inspect", false));
            if i.status.stoppable() {
                v.push(mk("stop", "Stop", false));
            }
            v.push(mk("purge", "Purge…", true));
        }
        ManagerRowKey::Workspace(_) => {
            v.push(mk("launch", "Launch", false));
            v.push(mk("edit", "Edit", false));
            v.push(mk("prewarm", "Prewarm", false));
            v.push(mk("delete", "Delete…", true));
        }
        ManagerRowKey::CurrentDirectory => {
            v.push(mk("launch", "Launch", false));
            if world.cwd_workspace().is_some() {
                v.push(mk("edit", "Edit", false));
            } else {
                v.push(mk("new", "Create workspace", false));
            }
        }
        ManagerRowKey::NewWorkspace => {
            v.push(mk("new", "Create workspace", false));
        }
    }
    v
}

pub fn row_layout(area: Rect, widths: &[u16], gap: u16) -> Vec<Rect> {
    let mut x = area.x;
    let mut out = Vec::new();
    for &w in widths {
        let w = w.min(area.right().saturating_sub(x));
        out.push(Rect::new(x, area.y, w, area.height.min(1)));
        x = x.saturating_add(w).saturating_add(gap);
    }
    out
}

pub use crate::manager_actions::inspect_facts;

const INSPECT_PANEL_PATCH: [(Part, StylePatch); 4] = [
    (
        Part::CONTAINER,
        StylePatch::new().set_bg(Role::Surface(Surface::Surface)),
    ),
    (
        Part::BORDER,
        StylePatch::new()
            .set_fg(Role::BorderSubtle)
            .set_bg(Role::Surface(Surface::Surface)),
    ),
    (
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Surface))
            .add(Modifier::BOLD),
    ),
    (
        Part::DETAIL,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::Surface(Surface::Surface)),
    ),
];

/// Inspect dialog displaying durable instance facts.
pub struct InspectDialog<'a> {
    pub id: Id,
    pub title: &'a str,
    pub facts: &'a [Fact],
    pub focused_index: usize,
}

impl<'a> InspectDialog<'a> {
    pub const fn new(id: Id, title: &'a str, facts: &'a [Fact]) -> Self {
        Self {
            id,
            title,
            facts,
            focused_index: 0,
        }
    }

    pub const fn focused_index(mut self, index: usize) -> Self {
        self.focused_index = index;
        self
    }

    pub fn layer_spec(id: Id) -> LayerSpec {
        LayerSpec::modal(id).size(LayerSize::Fixed(66, 14))
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.with_surface(Surface::Surface, |ui| {
            let palette = ManagerPalette::new(ui);
            let dialog_w = 66u16.min(area.width);
            let dialog_h = 14u16.min(area.height);
            let dialog_area = Rect::new(
                area.x.saturating_add((area.width.saturating_sub(dialog_w)) / 2),
                area.y.saturating_add((area.height.saturating_sub(dialog_h)) / 2),
                dialog_w,
                dialog_h,
            );

            Panel::new(self.id)
                .kind(PanelKind::Framed)
                .title(self.title)
                .meta("read-only")
                .patch_part(&INSPECT_PANEL_PATCH)
                .inner_inset(Insets::all(1))
                .draw(ui, dialog_area, |ui, body| {
                    let focus_bar_style = resolve_style(ui, Role::Focus, Role::Surface(Surface::Surface), true);
                    let label_style = palette.card_secondary;
                    let text_style = palette.card_primary;
                    let bold_style = palette.card_primary_bold;
                    let shortcut_style = palette.card_muted;

                    for (i, fact) in self.facts.iter().enumerate() {
                        let y = body.y.saturating_add(i as u16);
                        if y >= body.bottom() {
                            break;
                        }
                        let is_focused = i == self.focused_index;
                        ui.paint_str(Rect::new(body.x, y, 1, 1), " ", label_style);
                        if is_focused {
                            ui.paint_str(Rect::new(body.x.saturating_add(1), y, 1, 1), "▎", focus_bar_style);
                            ui.paint_str(Rect::new(body.x.saturating_add(2), y, 1, 1), " ", bold_style);
                            ui.paint_str(Rect::new(body.x.saturating_add(3), y, fact.label.len() as u16, 1), fact.label, label_style);

                            let val_start = body.x.saturating_add(3).saturating_add(fact.label.len() as u16);
                            let hint_start = body.right().saturating_sub(9);
                            let val_col = body.x.saturating_add(14);
                            let lead_spaces = val_col.saturating_sub(val_start);
                            let trail_spaces = hint_start.saturating_sub(val_col.saturating_add(fact.value.len() as u16));
                            let val_str = format!("{}{}{}", " ".repeat(lead_spaces as usize), fact.value, " ".repeat(trail_spaces as usize));
                            ui.paint_str(Rect::new(val_start, y, val_str.len() as u16, 1), &val_str, bold_style);

                            if fact.copyable {
                                ui.paint_str(Rect::new(hint_start, y, 6, 1), "y copy", shortcut_style);
                            } else {
                                ui.paint_str(Rect::new(hint_start, y, 6, 1), "      ", bold_style);
                            }
                            ui.paint_str(Rect::new(hint_start.saturating_add(6), y, 1, 1), " ", bold_style);
                            ui.paint_str(Rect::new(body.right().saturating_sub(2), y, 2, 1), "  ", label_style);
                        } else {
                            ui.paint_str(Rect::new(body.x.saturating_add(1), y, 1, 1), " ", resolve_style(ui, Role::Surface(Surface::Surface), Role::Surface(Surface::Surface), false));
                            ui.paint_str(Rect::new(body.x.saturating_add(2), y, 1, 1), " ", text_style);
                            ui.paint_str(Rect::new(body.x.saturating_add(3), y, fact.label.len() as u16, 1), fact.label, label_style);

                            let val_start = body.x.saturating_add(3).saturating_add(fact.label.len() as u16);
                            let val_end = body.right().saturating_sub(2);
                            let val_col = body.x.saturating_add(14);
                            let lead_spaces = val_col.saturating_sub(val_start);
                            let trail_spaces = val_end.saturating_sub(val_col.saturating_add(fact.value.len() as u16));
                            let val_str = format!("{}{}{}", " ".repeat(lead_spaces as usize), fact.value, " ".repeat(trail_spaces as usize));
                            ui.paint_str(Rect::new(val_start, y, val_str.len() as u16, 1), &val_str, text_style);

                            ui.paint_str(Rect::new(body.right().saturating_sub(2), y, 2, 1), "  ", label_style);
                        }
                    }

                    let btn_y = body.bottom().saturating_sub(1);
                    let btn_x = body.right().saturating_sub(9);
                    Button::new(INSPECT_CLOSE, "Close")
                        .variant(Variant::SECONDARY)
                        .draw(ui, Rect::new(btn_x, btn_y, 7, 1));
                    ui.paint_str(Rect::new(body.right().saturating_sub(2), btn_y, 2, 1), "  ", label_style);
                });
        });
    }
}

pub struct ManagerScreen;

impl ManagerScreen {
    pub fn draw(
        ui: &mut Ui<'_>,
        area: Rect,
        state: &ManagerState,
        world: &World,
        focused: bool,
    ) {
        let palette = ManagerPalette::new(ui);
        let full = ui.full();
        let stage = Rect::new(
            full.x,
            full.y.saturating_add(1),
            full.width,
            full.height.saturating_sub(2),
        );
        ui.fill(stage, palette.canvas);
        let rows = build_rows(world, state);
        let is_narrow = area.width < 100;
        let is_wide = area.width >= 150;

        if is_narrow {
            let summary_h = 6u16.min(area.height / 3);
            let tree_area = Rect::new(
                area.x,
                area.y,
                area.width,
                area.height.saturating_sub(summary_h + 1),
            );
            let summary_area = Rect::new(area.x, tree_area.bottom() + 1, area.width, summary_h);
            if state.detail_open() {
                Self::draw_detail(ui, area, state, world, focused, true, &palette);
            } else {
                Self::draw_tree(ui, tree_area, state, world, focused, &rows, &palette);
                Self::draw_summary(ui, summary_area, state, world, &palette);
            }
            return;
        }

        let split = SplitPane::new(SEAM, SplitAxis::Horizontal)
            .gap(2)
            .seam_end(1)
            .min_first(28)
            .min_second(40);
        let split_state = SplitPaneState::new(32);

        split.draw(ui, area, &split_state, |ui, left, right| {
            let tree_focused = focused && !state.detail_open();
            Self::draw_tree(ui, left, state, world, tree_focused, &rows, &palette);
            if is_wide {
                let detail_w = (right.width.saturating_sub(2) * 55) / 100;
                let detail_area = Rect::new(right.x, right.y, detail_w, right.height);
                let roster_area = Rect::new(
                    right.x + detail_w + 2,
                    right.y,
                    right.width.saturating_sub(detail_w + 2),
                    right.height,
                );
                Self::draw_detail(
                    ui,
                    detail_area,
                    state,
                    world,
                    focused && state.detail_open(),
                    false,
                    &palette,
                );
                Self::draw_roster(ui, roster_area, state, world, false, &palette);
            } else {
                Self::draw_detail(
                    ui,
                    right,
                    state,
                    world,
                    focused && state.detail_open(),
                    false,
                    &palette,
                );
            }
        });
    }

    fn draw_tree(
        ui: &mut Ui<'_>,
        area: Rect,
        state: &ManagerState,
        world: &World,
        focused: bool,
        rows: &[ManagerRow],
        palette: &ManagerPalette,
    ) {
        let running = world.running_count();
        let cursor_index = rows.iter().position(|r| {
            if let Some(c) = state.list.cursor() {
                c == ItemKey::text(&r.key.stable_key())
            } else {
                state.selected_row() == &r.key
            }
        });
        let mut scroll = ScrollState::new(rows.len());
        scroll.set_viewport(area.height.saturating_sub(2) as usize);
        if let Some(idx) = cursor_index {
            scroll.ensure_visible(idx);
        }
        let pos = position_label(&scroll);
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

        ui.register_control(TREE, area, Focusability::Focusable);

        Panel::new(TREE)
            .kind(PanelKind::Framed)
            .title("Workspaces")
            .meta(&meta)
            .focused(focused)
            .slot(Part::GUTTER, &|_ui, _cell| {})
            .patch_part(&FAINT_DETAIL_PATCH)
            .inner_inset(Insets {
                l: 2,
                t: 1,
                r: 2,
                b: 1,
            })
            .draw(ui, area, |ui, inner| {
                let has_sb = scroll.overflows();
                let row_w = inner.width.saturating_sub(u16::from(has_sb));
                let show_meta = row_w >= 44;
                for (k, i) in scroll.visible_range().enumerate() {
                    let y = inner.y + k as u16;
                    if i >= rows.len() {
                        break;
                    }
                    let row = &rows[i];
                    let is_selected = cursor_index == Some(i);
                    let is_focused = focused && is_selected;
                    let rect = Rect::new(inner.x, y, row_w, 1);
                    if is_focused {
                        ui.fill(rect, palette.primary_on_accent_tint_bold);
                        ui.paint_str(
                            Rect::new(rect.x, y, 1, 1),
                            "▎",
                            palette.accent_on_accent_tint_bold,
                        );
                    } else {
                        ui.fill(rect, palette.canvas);
                        ui.paint_str(Rect::new(rect.x, y, 1, 1), " ", palette.gutter_unfocused);
                    }
                    let mut x = rect.x + 2 + row.depth * 2;
                    let gs = if is_focused {
                        match row.glyph_tone {
                            Role::Danger => palette.danger_on_canvas,
                            Role::Fg(FgStep::Primary) => palette.primary_on_accent_tint_bold,
                            _ => palette.secondary_on_accent_tint,
                        }
                    } else {
                        match row.glyph_tone {
                            Role::Danger => palette.danger_on_canvas,
                            Role::Fg(FgStep::Primary) => palette.primary_on_canvas,
                            Role::Fg(FgStep::Muted) => palette.muted_on_canvas,
                            _ => palette.secondary_on_canvas,
                        }
                    };
                    ui.paint_str(Rect::new(x, y, 1, 1), row.glyph, gs);
                    x += 2;
                    let meta_w = if show_meta {
                        width(&row.meta) as u16
                    } else {
                        0
                    };
                    let trailing_w: u16 = if row.trailing.is_some() { 2 } else { 0 };
                    let avail = rect.right().saturating_sub(x + 1);
                    let lw = avail.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 0 } + trailing_w);
                    let label_style = if row.key == ManagerRowKey::NewWorkspace {
                        if is_focused {
                            palette.primary_on_accent_tint_bold
                        } else {
                            palette.secondary_on_canvas
                        }
                    } else if is_focused {
                        palette.accent_on_accent_tint_bold
                    } else if is_selected {
                        palette.accent_on_canvas
                    } else {
                        palette.primary_on_canvas
                    };
                    let label_text = fit(&truncate_middle(&row.label, lw), lw);
                    ui.paint_str(Rect::new(x, y, lw, 1), &label_text, label_style);
                    if meta_w > 0 && meta_w + 4 < avail {
                        let ms = if is_focused {
                            palette.muted_on_accent_tint
                        } else {
                            match row.meta_tone {
                                Role::Danger => palette.danger_on_canvas,
                                Role::Warning => palette.warning_on_canvas,
                                Role::Fg(FgStep::Secondary) => palette.secondary_on_canvas,
                                Role::Fg(FgStep::Faint) => palette.faint_on_canvas,
                                _ => palette.muted_on_canvas,
                            }
                        };
                        ui.paint_str(
                            Rect::new(rect.right().saturating_sub(meta_w + 1 + trailing_w), y, meta_w, 1),
                            &row.meta,
                            ms,
                        );
                    }
                    if let Some((g, tone)) = row.trailing {
                        let ts = match tone {
                            Role::Warning => palette.warning_on_canvas,
                            _ => palette.secondary_on_canvas,
                        };
                        ui.paint_str(Rect::new(rect.right().saturating_sub(2), y, 1, 1), g, ts);
                    }
                }
                if has_sb {
                    ui.scroll_edges(
                        Rect::new(inner.x, inner.y, (inner.right() - 1).saturating_sub(inner.x), inner.height),
                        &scroll,
                    );
                    let sb_rect = Rect::new(inner.right().saturating_sub(1), inner.y, 1, inner.height);
                    let track_len = usize::from(sb_rect.height);
                    let (thumb_start, thumb_len) = scroll.thumb(track_len);
                    for row in 0..track_len {
                        let y = sb_rect.y + row as u16;
                        let pos = Rect::new(sb_rect.x, y, 1, 1);
                        if row >= thumb_start && row < thumb_start + thumb_len {
                            ui.paint_str(pos, "┃", palette.primary_on_canvas);
                        } else {
                            ui.paint_str(pos, "│", palette.border_on_canvas);
                        }
                    }
                }
            });
    }

    fn draw_summary(
        ui: &mut Ui<'_>,
        area: Rect,
        state: &ManagerState,
        world: &World,
        palette: &ManagerPalette,
    ) {
        let title = match state.selected_row() {
            ManagerRowKey::CurrentDirectory => "Current directory".to_owned(),
            ManagerRowKey::Workspace(id) => world.workspace(*id).map(|x| x.name.clone()).unwrap_or_default(),
            ManagerRowKey::Instance(id) => format!("Instance {}", id.trim_start_matches("jk-")),
            ManagerRowKey::NewWorkspace => "New workspace".into(),
        };
        Panel::new(DETAIL)
            .kind(PanelKind::Card)
            .title(&title)
            .meta("Tab details")
            .draw(ui, area, |ui, inner| {
                let lines: Vec<(String, PaintStyle)> = match state.selected_row() {
                    ManagerRowKey::CurrentDirectory | ManagerRowKey::Workspace(_) => {
                        let ws = match state.selected_row() {
                            ManagerRowKey::Workspace(id) => world.workspace(*id),
                            _ => world.cwd_workspace(),
                        };
                        match ws {
                            Some(ws) => {
                                let mut l = vec![
                                    (
                                        format!(
                                            "{} · {} · {}",
                                            ws.workdir,
                                            plural(ws.mounts.len(), "mount", "mounts"),
                                            plural(ws.env_count(), "var", "vars")
                                        ),
                                        palette.card_secondary,
                                    ),
                                    (
                                        format!(
                                            "Roles {}{} · Auth {}",
                                            ws.roles
                                                .default
                                                .as_deref()
                                                .map(|d| format!("{} ★", role_label(world, d)))
                                                .unwrap_or("none".into()),
                                            match &ws.roles.allowed {
                                                AllowedRoles::All => " · all".to_owned(),
                                                AllowedRoles::Custom(list) =>
                                                    format!(" · {} allowed", list.len()),
                                            },
                                            world
                                                .account_for(Agent::ClaudeCode.provider(), Some(ws), None, None)
                                                .label(&world.accounts)
                                        ),
                                        palette.card_muted,
                                    ),
                                ];
                                l.extend(world.instances_of(Some(ws.id)).iter().take(2).map(|i| {
                                    (
                                        format!(
                                            "{} {}  {} · {} · {}",
                                            if i.status.is_live() { "◉" } else { "◌" },
                                            i.id.trim_start_matches("jk-"),
                                            role_label(world, &i.role),
                                            i.agent.label(),
                                            i.status.label()
                                        ),
                                        palette.card_secondary,
                                    )
                                }));
                                l
                            }
                            None => vec![(
                                format!(
                                    "{} · not saved · Enter launches, n creates a workspace",
                                    world.tilde(&world.cwd)
                                ),
                                palette.card_muted,
                            )],
                        }
                    }
                    ManagerRowKey::Instance(id) => match world.instance(id) {
                        Some(i) => vec![
                            (
                                format!(
                                    "{} · {} · {} · {}",
                                    role_label(world, &i.role),
                                    i.agent.label(),
                                    i.status.label(),
                                    i.dirty_summary()
                                ),
                                palette.card_secondary,
                            ),
                            (
                                match &i.daemon {
                                    DaemonSnapshot::Tabs(tabs) => format!(
                                        "daemon · {} · {}",
                                        plural(tabs.len(), "tab", "tabs"),
                                        plural(
                                            tabs.iter().map(|t| t.panes.len()).sum(),
                                            "pane",
                                            "panes"
                                        )
                                    ),
                                    DaemonSnapshot::NoTabs => "daemon reports no tabs".into(),
                                    DaemonSnapshot::Unavailable => "daemon unavailable".into(),
                                },
                                palette.card_muted,
                            ),
                        ],
                        None => vec![],
                    },
                    ManagerRowKey::NewWorkspace => vec![(
                        "Enter starts the five-step create chain".into(),
                        palette.card_muted,
                    )],
                };
                for (i, (text, style)) in lines.iter().enumerate() {
                    let y = inner.y + i as u16;
                    if y >= inner.bottom() {
                        break;
                    }
                    let t = truncate(text, inner.width);
                    ui.paint_str(Rect::new(inner.x, y, inner.width, 1), &t, *style);
                }
            });
    }

    fn draw_detail(
        ui: &mut Ui<'_>,
        area: Rect,
        state: &ManagerState,
        world: &World,
        focused: bool,
        as_drawer: bool,
        palette: &ManagerPalette,
    ) {
        let actions = rebuild_actions(world, state.selected_row());
        let (title, scope_word): (String, String) = match state.selected_row() {
            ManagerRowKey::CurrentDirectory => (
                match world.cwd_workspace() {
                    Some(ws) => format!("Current directory · {}", ws.name),
                    None => "Current directory".into(),
                },
                match world.cwd_workspace() {
                    Some(_) => "saved workspace".into(),
                    None => "not saved".into(),
                },
            ),
            ManagerRowKey::Workspace(id) => {
                let ws = world.workspace(*id);
                let n = world
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
            ManagerRowKey::Instance(id) => {
                let i = world.instance(id);
                (
                    format!(
                        "{} · {}",
                        id.trim_start_matches("jk-"),
                        i.and_then(|x| x.workspace)
                            .and_then(|x| world.workspace(x))
                            .map(|x| x.name.as_str())
                            .unwrap_or("current directory")
                    ),
                    format!("instance · {}", i.map(|x| x.status.label()).unwrap_or("?")),
                )
            }
            ManagerRowKey::NewWorkspace => ("New workspace".into(), "create".into()),
        };

        let kind = if as_drawer {
            PanelKind::Framed
        } else {
            PanelKind::Card
        };

        let mut panel = Panel::new(DETAIL)
            .kind(kind)
            .title(&title)
            .meta(&scope_word)
            .focused(focused)
            .patch_part(&FAINT_DETAIL_PATCH);
        if kind == PanelKind::Card {
            panel = panel.inner_inset(Insets {
                l: 1,
                t: 2,
                r: 2,
                b: 1,
            });
        }
        panel.draw(ui, area, |ui, inner| {
            let cx = if kind == PanelKind::Card { inner.x + 1 } else { inner.x };
            let cw = if kind == PanelKind::Card { inner.width.saturating_sub(1) } else { inner.width };
            let mut y = inner.y;
                match state.selected_row() {
                    ManagerRowKey::CurrentDirectory | ManagerRowKey::Workspace(_) => {
                        let ws = match state.selected_row() {
                            ManagerRowKey::Workspace(id) => world.workspace(*id),
                            _ => world.cwd_workspace(),
                        };
                        match ws {
                            Some(ws) => {
                                let mounts_text = if ws.mounts.is_empty() {
                                    "none".into()
                                } else {
                                    ws.mounts
                                        .iter()
                                        .map(|m| {
                                            format!(
                                                "{} · {} {}",
                                                world.tilde(m.source_label()),
                                                m.mode_label(),
                                                m.isolation.label().to_lowercase()
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                };
                                let roles_text = format!(
                                    "{}{}",
                                    ws.roles
                                        .default
                                        .as_ref()
                                        .map(|d| format!("{} ★ · ", role_label(world, d)))
                                        .unwrap_or_default(),
                                    match &ws.roles.allowed {
                                        AllowedRoles::All =>
                                            format!("allowed all ({} in registry)", world.roles.len()),
                                        AllowedRoles::Custom(l) =>
                                            format!("allowed {} of {}", l.len(), world.roles.len()),
                                    }
                                );
                                let envs_text = format!(
                                    "{} · {} [op]",
                                    plural(ws.env_count(), "var", "vars"),
                                    ws.env
                                        .iter()
                                        .chain(ws.role_env.values().flatten())
                                        .filter(|e| matches!(e.value, EnvValue::OnePassword(_)))
                                        .count()
                                );
                                let effective = ws.effective_accounts(&world.accounts);
                                let mut acct_lines: Vec<String> = effective
                                    .iter()
                                    .map(|e| {
                                        let name = world
                                            .accounts
                                            .get(&e.id)
                                            .map(|a| a.title())
                                            .unwrap_or_else(|| e.id.clone());
                                        let mut line = name;
                                        if e.preferred {
                                            line.push_str(" ★");
                                        }
                                        if !e.usable.is_ready() {
                                            line.push_str(&format!(" · {}", e.usable.label()));
                                        }
                                        line
                                    })
                                    .collect();
                                if acct_lines.is_empty() {
                                    acct_lines.push("none active · enable one in the editor".into());
                                }
                                let accounts_text = acct_lines.join(" · ");
                                let policies_text = format!(
                                    "git pull {} · keep awake {} · dirty exit {}",
                                    if ws.git_pull { "enabled" } else { "disabled" },
                                    if ws.keep_awake { "on" } else { "off" },
                                    ws.dirty_policy.label()
                                );

                                let props_rows = vec![
                                    PropsRow::new(ItemKey::Index(0), "Working dir", &ws.workdir),
                                    PropsRow::new(ItemKey::Index(1), "Mounts", &mounts_text).wrap(),
                                    PropsRow::new(ItemKey::Index(2), "Roles", &roles_text),
                                    PropsRow::new(ItemKey::Index(3), "Environments", &envs_text),
                                    PropsRow::new(ItemKey::Index(4), "Accounts", &accounts_text).wrap(),
                                    PropsRow::new(ItemKey::Index(5), "Policies", &policies_text),
                                ];
                                let used = Props::rich(&props_rows).draw(
                                    ui,
                                    Rect::new(cx, y, cw, inner.height),
                                );
                                y += used.height + 1;

                                let kids = world.instances_of(Some(ws.id));
                                if !kids.is_empty() {
                                    ui.paint_str(
                                        Rect::new(cx, y, cw, 1),
                                        "Instances",
                                        palette.card_secondary_bold,
                                    );
                                    let meta = match world.daemon_health {
                                        DaemonHealth::Healthy => {
                                            format!("daemon · {}", world.clock.ago(world.last_refresh_secs))
                                        }
                                        DaemonHealth::Stale => {
                                            format!("▲ daemon stale · {}", world.clock.ago(world.last_refresh_secs))
                                        }
                                    };
                                    let mw = width(&meta) as u16;
                                    if y < inner.bottom() && inner.width > mw + 12 {
                                        ui.paint_str(
                                            Rect::new(inner.right().saturating_sub(mw), y, mw, 1),
                                            &meta,
                                            palette.card_faint,
                                        );
                                    }
                                    y += 1;
                                    for i in kids {
                                        let line = format!(
                                            "{} {}  {} · {} · {}",
                                            match i.status {
                                                InstanceStatus::Running => "◉",
                                                InstanceStatus::Crashed | InstanceStatus::FailedSetup => "!",
                                                _ => "◌",
                                            },
                                            i.id.trim_start_matches("jk-"),
                                            role_label(world, &i.role),
                                            i.agent.label(),
                                            i.status.label()
                                        );
                                        let style = if i.status.is_live() {
                                            palette.card_primary
                                        } else {
                                            palette.card_secondary
                                        };
                                        ui.paint_str(
                                            Rect::new(cx, y, cw, 1),
                                            &truncate(&line, cw),
                                            style,
                                        );
                                        y += 1;
                                    }
                                }
                            }
                            None => {
                                let empty_state = EmptyState::Empty {
                                    title: "Create a workspace from this directory.",
                                    hint: Some(&format!(
                                        "{} is mounted at its own path inside the Construct. Enter launches with defaults; n creates a saved workspace.",
                                        world.tilde(&world.cwd)
                                    )),
                                };
                                Empty::new(DETAIL.sub("empty"), empty_state).draw(
                                    ui,
                                    Rect::new(
                                        cx,
                                        inner.y,
                                        cw,
                                        inner.height.saturating_sub(3),
                                    ),
                                );
                                y = inner.bottom().saturating_sub(2);
                            }
                        }
                    }
                    ManagerRowKey::NewWorkspace => {
                        let empty_state = EmptyState::Empty {
                            title: "New workspace",
                            hint: Some("Enter starts the five-step create chain: source, destination, working directory, name, then the editor."),
                        };
                        Empty::new(DETAIL.sub("empty"), empty_state).draw(
                            ui,
                            Rect::new(
                                cx,
                                inner.y,
                                cw,
                                inner.height.saturating_sub(3),
                            ),
                        );
                        y = inner.bottom().saturating_sub(2);
                    }
                    ManagerRowKey::Instance(id) => {
                        if let Some(i) = world.instance(id) {
                            let ws = i.workspace.and_then(|x| world.workspace(x));
                            let acc = world.account_for(i.agent.provider(), ws, Some(&i.role), None);
                            let ws_label = format!(
                                "{} › role {}",
                                ws.map(|x| x.name.as_str()).unwrap_or("current directory"),
                                role_label(world, &i.role)
                            );
                            let agent_label = format!(
                                "{} · account {} ({})",
                                i.agent.label(),
                                acc.label(&world.accounts),
                                acc.level_label()
                            );
                            let started_label = format!(
                                "{} · last seen {}",
                                world.clock.ago(i.created_secs),
                                world.clock.ago(i.last_seen_secs)
                            );
                            let lifecycle_label = format!("{} · {}", i.status.label(), i.status.description());
                            let container_id = i.container_id();
                            let dirty_summary = i.dirty_summary();

                            let mut props_rows = vec![
                                PropsRow::new(ItemKey::Index(0), "Workspace", &ws_label),
                                PropsRow::new(ItemKey::Index(1), "Agent", &agent_label),
                                PropsRow::new(ItemKey::Index(2), "Container", &container_id),
                                PropsRow::new(ItemKey::Index(3), "Started", &started_label),
                                PropsRow::new(ItemKey::Index(4), "Lifecycle", &lifecycle_label).wrap(),
                                PropsRow::new(ItemKey::Index(5), "Working tree", &dirty_summary),
                            ];
                            let branch_label = i.branch.as_ref().map(|b| match &i.pr {
                                Some((n, title)) => format!("{b} · PR #{n} · {title}"),
                                None => b.clone(),
                            });
                            if let Some(ref bl) = branch_label {
                                props_rows.push(PropsRow::new(ItemKey::Index(6), "Branch", bl));
                            }
                            let used = Props::rich(&props_rows).draw(
                                ui,
                                Rect::new(cx, y, cw, inner.height),
                            );
                            y += used.height + 1;

                            ui.paint_str(
                                Rect::new(cx, y, cw, 1),
                                "Live topology",
                                palette.card_secondary_bold,
                            );
                            let src = match &i.daemon {
                                DaemonSnapshot::Unavailable => "unavailable".to_owned(),
                                _ => format!("daemon · {}", world.clock.ago(i.last_seen_secs)),
                            };
                            let sw = width(&src) as u16;
                            if y < inner.bottom() && inner.width > sw + 16 {
                                ui.paint_str(
                                    Rect::new(inner.right().saturating_sub(sw), y, sw, 1),
                                    &src,
                                    palette.card_faint,
                                );
                            }
                            y += 1;

                            let detail_rows = build_detail(world, id);
                            let list_top = y;
                            let rows_avail = inner.bottom().saturating_sub(y + 2) as usize;
                            let mut detail_scroll = ScrollState::new(detail_rows.len());
                            detail_scroll.set_viewport(rows_avail);

                            for (k, ri) in detail_scroll.visible_range().enumerate() {
                                let yy = list_top + k as u16;
                                if yy >= inner.bottom().saturating_sub(2) {
                                    break;
                                }
                                let row = &detail_rows[ri];
                                match row {
                                    DetailRow::Pane { text, .. } => {
                                        ui.paint_str(
                                            Rect::new(cx + 2, yy, cw.saturating_sub(3), 1),
                                            &truncate(text, cw.saturating_sub(3)),
                                            palette.card_primary,
                                        );
                                    }
                                    DetailRow::Text(text, tone) => {
                                        let style = match tone {
                                            Role::Danger => palette.card_danger,
                                            _ => palette.card_muted,
                                        };
                                        ui.paint_str(
                                            Rect::new(cx, yy, cw, 1),
                                            &truncate(text, cw),
                                            style,
                                        );
                                    }
                                    DetailRow::Session(text) => {
                                        ui.paint_str(
                                            Rect::new(cx, yy, cw, 1),
                                            &truncate(text, cw),
                                            palette.card_secondary,
                                        );
                                    }
                                    DetailRow::Blank => {
                                        ui.paint_str(
                                            Rect::new(cx, yy, cw, 1),
                                            "Sessions",
                                            palette.card_secondary_bold,
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
                                            ui.paint_str(
                                                Rect::new(inner.right().saturating_sub(mw), yy, mw, 1),
                                                &m,
                                                palette.card_faint,
                                            );
                                        }
                                    }
                                }
                            }
                            y = inner.bottom().saturating_sub(2);
                        }
                    }
                }

                // Action buttons at the bottom
                let ay = inner.bottom().saturating_sub(1).max(y);
                if ay < inner.bottom() && !actions.is_empty() {
                    let widths: Vec<u16> = actions
                        .iter()
                        .map(|b| (width(b.label) + 2) as u16)
                        .collect();
                    let rects = row_layout(Rect::new(inner.x, ay, inner.width, 1), &widths, 2);
                    for (b, r) in actions.iter().zip(rects) {
                        let variant = if b.danger {
                            Variant::DANGER
                        } else {
                            Variant::SECONDARY
                        };
                        Button::new(b.id, b.label)
                            .variant(variant)
                            .draw(ui, r);
                    }
                }
            });
    }

    fn draw_roster(
        ui: &mut Ui<'_>,
        area: Rect,
        state: &ManagerState,
        world: &World,
        focused: bool,
        palette: &ManagerPalette,
    ) {
        let running = world.running();
        let meta = plural(running.len(), "instance", "instances");
        Panel::new(ROSTER)
            .kind(PanelKind::Card)
            .title("Running")
            .meta(&meta)
            .focused(focused)
            .draw(ui, area, |ui, inner| {
                let mut y = inner.y;
                for i in &running {
                    if y + 2 >= inner.bottom() {
                        break;
                    }
                    let ws = i
                        .workspace
                        .and_then(|x| world.workspace(x))
                        .map(|x| x.name.as_str())
                        .unwrap_or("current directory");
                    let acc = world.account_for(
                        i.agent.provider(),
                        i.workspace.and_then(|x| world.workspace(x)),
                        Some(&i.role),
                        None,
                    );
                    let is_selected = state.selected_row() == &ManagerRowKey::Instance(i.id.clone());
                    let row_rect = Rect::new(inner.x.saturating_sub(1), y, inner.width + 1, 3);
                    if is_selected {
                        ui.fill(row_rect, palette.primary_on_accent_tint_bold);
                    }
                    let header = format!(
                        "◉ {}  {ws} · {} · {}",
                        i.id.trim_start_matches("jk-"),
                        role_label(world, &i.role),
                        i.agent.label()
                    );
                    let style = if is_selected {
                        palette.accent_on_accent_tint_bold
                    } else {
                        palette.card_primary
                    };
                    ui.paint_str(
                        Rect::new(inner.x, y, inner.width, 1),
                        &truncate(&header, inner.width),
                        style,
                    );
                    let live = match &i.daemon {
                        DaemonSnapshot::Tabs(tabs) => format!(
                            "running {} · {} · {}",
                            format_duration((world.now_secs() - i.created_secs).max(0) as u64),
                            plural(tabs.len(), "tab", "tabs"),
                            plural(tabs.iter().map(|t| t.panes.len()).sum(), "pane", "panes")
                        ),
                        _ => format!(
                            "running {} · daemon unavailable",
                            format_duration((world.now_secs() - i.created_secs).max(0) as u64)
                        ),
                    };
                    ui.paint_str(
                        Rect::new(inner.x + 3, y + 1, inner.width.saturating_sub(3), 1),
                        &truncate(&live, inner.width.saturating_sub(3)),
                        if is_selected {
                            palette.muted_on_accent_tint
                        } else {
                            palette.card_muted
                        },
                    );
                    let acc_line = format!("account {}", acc.label(&world.accounts));
                    ui.paint_str(
                        Rect::new(inner.x + 3, y + 2, inner.width.saturating_sub(3), 1),
                        &truncate(&acc_line, inner.width.saturating_sub(3)),
                        if is_selected {
                            palette.muted_on_accent_tint
                        } else {
                            palette.card_muted
                        },
                    );
                    y += 4;
                }

                let preserved: Vec<_> = world
                    .instances
                    .iter()
                    .filter(|i| i.status.dirty() || i.status == InstanceStatus::RestoreAvailable)
                    .collect();
                if !preserved.is_empty() && y + 2 < inner.bottom() {
                    y += 1;
                    ui.paint_str(
                        Rect::new(inner.x, y, 10, 1),
                        "Preserved",
                        palette.card_secondary_bold,
                    );
                    let m = plural(preserved.len(), "record", "records");
                    let mw = width(&m) as u16;
                    if inner.width > mw + 12 {
                        ui.paint_str(
                            Rect::new(inner.right().saturating_sub(mw), y, mw, 1),
                            &m,
                            palette.card_faint,
                        );
                    }
                    y += 1;
                    for i in preserved {
                        if y >= inner.bottom() {
                            break;
                        }
                        let ws = i
                            .workspace
                            .and_then(|x| world.workspace(x))
                            .map(|x| x.name.as_str())
                            .unwrap_or("current directory");
                        let line = format!(
                            "◌ {}  {ws} · {} · {}",
                            i.id.trim_start_matches("jk-"),
                            role_label(world, &i.role),
                            i.agent.label()
                        );
                        ui.paint_str(
                            Rect::new(inner.x, y, inner.width, 1),
                            &truncate(&line, inner.width),
                            palette.card_secondary,
                        );
                        y += 1;
                    }
                }
            });
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use jackin_preview_domain::account::{Account, CredentialSource, DetectedKind, Lifecycle};
    use jackin_preview_domain::agent::Provider;
    use jackin_preview_domain::scenario::Scenario;
    use jackin_preview_sim::world::world_for;

    #[test]
    fn tree_selection_keeps_stable_row_identity() {
        let mut state = ManagerState::default();
        assert_eq!(state.selected_row(), &ManagerRowKey::CurrentDirectory);

        state.select_row(ManagerRowKey::Workspace(7));
        assert_eq!(state.selected(), Some(7));
        assert_eq!(state.selected_row(), &ManagerRowKey::Workspace(7));
        assert_eq!(state.selected_row().stable_key(), "workspace:7");

        state.select_row(ManagerRowKey::Instance("run-7".into()));
        assert_eq!(state.selected(), None);
        assert_eq!(state.selected_row().stable_key(), "instance:run-7");
    }

    #[test]
    fn launch_candidates_omit_unconfigured_agents() {
        let mut world = world_for(Scenario::FirstUse);
        let mut account = Account::registered(
            "acct-only",
            "Only",
            Provider::Anthropic,
            CredentialSource::LocalFolder {
                path: "~/.claude".into(),
                detected: DetectedKind::ClaudeOAuthProfile,
            },
        );
        account.default_for_provider = true;
        world.accounts.insert(account);

        let candidates = ManagerState::launch_candidates(&world, None, None);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].agent, Agent::ClaudeCode);
        assert!(candidates[0].blocked.is_some());
        assert!(!candidates[0].is_ready());
    }

    #[test]
    fn launch_candidates_preserve_ready_account_selection() {
        let mut world = world_for(Scenario::FirstUse);
        let mut account = Account::registered(
            "acct-ready",
            "Ready",
            Provider::Anthropic,
            CredentialSource::LocalFolder {
                path: "~/.claude".into(),
                detected: DetectedKind::ClaudeOAuthProfile,
            },
        );
        account.lifecycle = Lifecycle::Available;
        account.default_for_provider = true;
        world.accounts.insert(account);

        let candidates = ManagerState::launch_candidates(&world, None, None);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].agent, Agent::ClaudeCode);
        assert_eq!(candidates[0].account.as_deref(), Some("acct-ready"));
        assert_eq!(candidates[0].blocked, None);
        assert!(candidates[0].is_ready());
    }
}
