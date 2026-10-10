//! Settings route state and trust-tab presentation.

use std::collections::BTreeMap;

use jackin_preview_domain::agent::{Agent, AuthMode};
use jackin_preview_domain::workspace::{EnvValue, Mount, mask};
use jackin_preview_sim::world::{TrustRow, World};
use ratatui::layout::Rect;
use termrock::ColorLevel;
use termrock::author::{
    Family, FgStep, Modifier, PaintStyle, Part, Role, StateFlags, StyleDefaults, StylePatch,
    Surface, Ui, Variant,
};
use termrock::controls::{Button, Checkbox, Panel};
use termrock::layout::Track;
use termrock::navigation::{List, ListState, Tabs, TabsState};
use termrock::{
    Action, ActionKey, Anchor, Backdrop, Dialog, DialogState, FrameRead, Hint, HintKey, HintLayer,
    Id, ItemKey, LayerSize, Props, PropsRow, ScreenAlign, backdrop_area, resolve_anchor,
    truncate, width,
};

/// Settings root.
pub const ROOT: Id = Id::root("jackin.settings");
/// Settings form namespace.
pub const FORM: Id = ROOT.sub("form");
/// Settings tab strip.
pub const TABS: Id = ROOT.sub("tabs");
/// Cancel action.
pub const CANCEL: Id = FORM.sub("cancel");
/// Save action.
pub const SAVE: Id = FORM.sub("save");
/// Trust body list.
pub const TRUST: Id = FORM.sub("trust");
/// General body focus target.
pub const GENERAL_BODY: Id = FORM.sub("general");
/// Co-author trailer checkbox.
pub const COAUTHOR: Id = FORM.sub("coauthor");
/// DCO sign-off checkbox.
pub const DCO: Id = FORM.sub("dco");
/// Save-preview dialog.
pub const PREVIEW: Id = ROOT.sub("preview");
/// Cancel action inside the save-preview dialog (the focused control).
pub const PREVIEW_CANCEL: Id = PREVIEW.part(Part::ACTIONS).index(0);
/// Mounts body focus target.
pub const MOUNTS_BODY: Id = FORM.sub("mounts");
/// Environments body focus target.
pub const ENV_BODY: Id = FORM.sub("env");
/// Wide-environments inspector card.
pub const ENV_CARD: Id = FORM.sub("env-card");
/// Agents body focus target.
pub const AGENTS_BODY: Id = FORM.sub("agents");

/// Settings tab names in tab order: General, Mounts, Environments, Agents,
/// Trust (tag `screens/settings.rs` `TAB_NAMES`).
pub const TAB_NAMES: [&str; 5] = ["General", "Mounts", "Environments", "Agents", "Trust"];

/// Which settings region owns the keyboard focus (tag focus model: `TABS`
/// shows the jump footer, the body shows per-tab hints, the Cancel/Save…
/// buttons show the choose footer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsFocus {
    /// The tab strip owns focus.
    #[default]
    Tabs,
    /// The active tab body owns focus.
    Body,
    /// The Cancel/Save… buttons own focus.
    Buttons,
}

/// Settings draft and save lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsState {
    /// Whether the settings draft has unsaved changes.
    pub dirty: bool,
    /// Number of save attempts made for the current draft.
    pub save_attempts: u8,
    /// Latest save error, if one occurred.
    pub save_error: Option<String>,
    /// Which settings region owns the keyboard focus.
    pub focus: SettingsFocus,
    /// Trust body cursor row.
    pub trust_cursor: usize,
    /// Pending trust edits by row index (index → pending trusted flag).
    /// Empty means the trust draft matches the world.
    pub trust_overrides: BTreeMap<usize, bool>,
}

impl SettingsState {
    /// Record a user edit and retain it across a failed save.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.save_error = None;
    }

    /// Begin a new draft lifecycle, clearing attempts from the prior draft.
    pub fn begin_draft(&mut self) {
        self.dirty = true;
        self.save_attempts = 0;
        self.save_error = None;
    }

    /// Whether the current draft can be discarded without confirmation.
    pub const fn is_clean(&self) -> bool {
        !self.dirty
    }

    /// Clear a displayed save error while retaining the draft.
    pub fn clear_error(&mut self) {
        self.save_error = None;
    }

    /// Effective trusted flag for one trust row: the pending override when
    /// present, otherwise the world value.
    pub fn trust_effective(&self, original: &[TrustRow], index: usize) -> bool {
        self.trust_overrides
            .get(&index)
            .copied()
            .unwrap_or_else(|| original.get(index).is_some_and(|row| row.trusted))
    }

    /// Pending trust edits (tag `change_count`, trust portion): overrides
    /// that differ from the world. Out-of-range overrides never count.
    pub fn trust_change_count(&self, original: &[TrustRow]) -> usize {
        self.trust_overrides
            .iter()
            .filter(|(index, pending)| {
                original
                    .get(**index)
                    .is_some_and(|row| row.trusted != **pending)
            })
            .count()
    }

    /// Whether the trust draft differs from the world.
    pub fn trust_dirty(&self, original: &[TrustRow]) -> bool {
        self.trust_change_count(original) > 0
    }

    /// Move the trust cursor by `delta`, clamped to the world rows.
    pub fn move_trust_cursor(&mut self, delta: i32, len: usize) {
        if len == 0 {
            self.trust_cursor = 0;
            return;
        }
        let next = self.trust_cursor as i32 + delta;
        self.trust_cursor = next.clamp(0, len as i32 - 1) as usize;
    }

    /// Flip the trusted flag at the cursor (tag `trust_key` space/enter).
    /// Toggling back to the world value drops the override. Returns the
    /// source and the new flag for the row status, or `None` when the
    /// cursor is out of range.
    pub fn toggle_trust(&mut self, original: &[TrustRow]) -> Option<(String, bool)> {
        let row = original.get(self.trust_cursor)?;
        let next = !self.trust_effective(original, self.trust_cursor);
        if next == row.trusted {
            self.trust_overrides.remove(&self.trust_cursor);
        } else {
            self.trust_overrides.insert(self.trust_cursor, next);
        }
        self.sync_dirty(original);
        Some((row.source.clone(), next))
    }

    /// Recompute the draft flag from the pending trust edits.
    pub fn sync_dirty(&mut self, original: &[TrustRow]) {
        self.dirty = self.trust_dirty(original);
        if self.dirty {
            self.save_error = None;
        }
    }

    /// Drop all pending trust edits (after a confirmed save).
    pub fn clear_trust(&mut self) {
        self.trust_overrides.clear();
        self.dirty = false;
        self.save_error = None;
    }

    /// Record one save attempt and return whether the draft should remain.
    pub fn attempt_save(&mut self, fails: bool) -> bool {
        self.save_attempts = self.save_attempts.saturating_add(1);
        if fails && self.save_attempts == 1 {
            self.dirty = true;
            self.save_error = Some("Settings error · host rejected the update".into());
            true
        } else {
            self.dirty = false;
            self.save_error = None;
            false
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UI Composition: SettingsScreen
// ─────────────────────────────────────────────────────────────────────────────

/// `n` rendered with the singular or plural noun (tag `plural`).
fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Resolve a canvas paint style (accounts palette pattern).
fn env_var<'a>(
    world: &'a World,
    scope: &str,
    key: &str,
) -> Option<&'a jackin_preview_domain::workspace::EnvVar> {
    let vars = if let Some(role) = scope.strip_prefix("role ") {
        world.global.role_env.get(role)?
    } else {
        &world.global.env
    };
    vars.iter().find(|var| var.key == key)
}

fn resolve_style(ui: &Ui<'_>, fg: Role, bold: bool) -> PaintStyle {
    let mut patch = StylePatch::new()
        .set_fg(fg)
        .set_bg(Role::Surface(Surface::Canvas));
    if bold {
        patch = patch.add(Modifier::BOLD);
    } else {
        patch = patch.remove(Modifier::BOLD);
    }
    ui.style_defaults(
        Family::PANEL,
        Variant::DEFAULT,
        Part::custom("jackin.settings.style"),
        StateFlags::empty(),
        StyleDefaults::new(patch),
        None,
    )
    .style
}

fn mount_widths(avail: u16) -> [u16; 6] {
    let fixed = 14 + 4 + 9 + 6;
    if avail >= 100 {
        let rest = avail.saturating_sub(fixed + 10);
        let dest = (rest * 45 / 100).max(20);
        [dest, 14, 4, 9, 6, rest.saturating_sub(dest)]
    } else if avail >= 70 {
        [avail.saturating_sub(fixed + 8), 14, 4, 9, 6, 0]
    } else {
        [avail.saturating_sub(14 + 4 + 9 + 6), 14, 4, 9, 0, 0]
    }
}

fn env_widths(avail: u16) -> [u16; 4] {
    let key = 18.min(avail / 3);
    let value = if avail >= 100 { 22 } else { 16 };
    [key, 14, value, avail.saturating_sub(key + 14 + value + 6)]
}

fn pad_cell(text: &str, columns: u16) -> String {
    if columns == 0 {
        return String::new();
    }
    let fitted = truncate(text, columns);
    let gap = usize::from(columns).saturating_sub(usize::from(width(&fitted)));
    format!("{fitted}{}", " ".repeat(gap))
}

/// Tracks for a config table whose columns start at `rect.x + 6` with a
/// two-cell gap, skipping hidden (`0`) columns. The row callback already
/// starts after List's gutter and marker, so cell 0 is the change glyph.
fn env_section_header(row: &EnvLine) -> bool {
    row.header
}

fn content_tracks(widths: &[u16]) -> (Vec<Track>, Vec<usize>) {
    let mut tracks = vec![Track::Fixed(1), Track::Fixed(2)];
    let mut cells = Vec::with_capacity(widths.len());
    let mut seen = false;
    for width in widths {
        if *width == 0 {
            cells.push(usize::MAX);
            continue;
        }
        if seen {
            tracks.push(Track::Fixed(2));
        }
        seen = true;
        cells.push(tracks.len());
        tracks.push(Track::Fixed(*width));
    }
    (tracks, cells)
}

fn auth_mode_label(mode: AuthMode) -> &'static str {
    match mode {
        AuthMode::Sync => "sync",
        AuthMode::ApiKey => "api key",
        AuthMode::OAuthToken => "oauth token",
        AuthMode::Ignore => "ignore",
    }
}

struct MountRow {
    index: usize,
    add: bool,
    cells: [String; 6],
}

impl MountRow {
    fn from_mount(mount: &Mount, world: &World) -> Self {
        let kind = match &mount.source {
            jackin_preview_domain::workspace::MountSource::Git(_) => "git",
            jackin_preview_domain::workspace::MountSource::Host(_) => "host",
        };
        Self {
            index: 0,
            add: false,
            cells: [
                mount.destination.clone(),
                mount.scope.label(),
                mount.mode_label().to_owned(),
                mount.isolation.label().to_lowercase(),
                kind.to_owned(),
                world.tilde(mount.source_label()),
            ],
        }
    }

    fn add() -> Self {
        Self {
            index: usize::MAX,
            add: true,
            cells: [
                "+ Add mount".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
        }
    }
}

struct EnvLine {
    index: usize,
    header: bool,
    /// Role sections draw a disclosure at `rect.x + 2`.
    fold: bool,
    add: bool,
    cursor: bool,
    title: String,
    scope: String,
    value: String,
    source: String,
    meta: String,
}

fn env_value(var: &jackin_preview_domain::workspace::EnvVar) -> (String, String) {
    match &var.value {
        EnvValue::Plain(text) => (mask(text), "plain".into()),
        EnvValue::OnePassword(reference) => {
            ("*".repeat(16), format!("[op] {}", reference.display_path()))
        }
        EnvValue::HostEnv(name) => (format!("${name}"), "host env".into()),
    }
}

fn env_rows(world: &World, _widths: &[u16; 4]) -> Vec<EnvLine> {
    let mut rows = Vec::new();
    let push = |rows: &mut Vec<EnvLine>, mut row: EnvLine| {
        row.index = rows.len();
        rows.push(row);
    };
    push(
        &mut rows,
        EnvLine {
            index: 0,
            header: true,
            fold: false,
            add: false,
            cursor: false,
            title: "Global".into(),
            scope: String::new(),
            value: String::new(),
            source: String::new(),
            meta: plural(world.global.env.len(), "var", "vars"),
        },
    );
    for var in &world.global.env {
        let (value, source) = env_value(var);
        push(
            &mut rows,
            EnvLine {
                index: 0,
                header: false,
                fold: false,
                add: false,
                cursor: false,
                title: var.key.clone(),
                scope: "global".into(),
                value,
                source,
                meta: String::new(),
            },
        );
    }
    push(
        &mut rows,
        EnvLine {
            index: 0,
            header: false,
            fold: false,
            add: true,
            cursor: false,
            title: "+ Add environment variable".into(),
            scope: String::new(),
            value: String::new(),
            source: String::new(),
            meta: String::new(),
        },
    );
    let configured = world.global.role_env.len();
    push(
        &mut rows,
        EnvLine {
            index: 0,
            header: true,
            fold: false,
            add: false,
            cursor: false,
            title: "Role overrides".into(),
            scope: String::new(),
            value: String::new(),
            source: String::new(),
            meta: format!(
                "{configured} configured · {} in the registry",
                world.roles.len()
            ),
        },
    );
    for (role, vars) in &world.global.role_env {
        push(
            &mut rows,
            EnvLine {
                index: 0,
                header: true,
                fold: true,
                add: false,
                cursor: false,
                title: format!("Role: {role}"),
                scope: String::new(),
                value: String::new(),
                source: String::new(),
                meta: plural(vars.len(), "var", "vars"),
            },
        );
        for var in vars {
            let (value, source) = env_value(var);
            push(
                &mut rows,
                EnvLine {
                    index: 0,
                    header: false,
                    fold: false,
                    add: false,
                    cursor: false,
                    title: var.key.clone(),
                    scope: format!("role {role}"),
                    value,
                    source,
                    meta: String::new(),
                },
            );
        }
        push(
            &mut rows,
            EnvLine {
                index: 0,
                header: false,
                fold: false,
                add: true,
                cursor: false,
                title: format!("+ Add {role} environment variable"),
                scope: String::new(),
                value: String::new(),
                source: String::new(),
                meta: String::new(),
            },
        );
    }
    push(
        &mut rows,
        EnvLine {
            index: 0,
            header: false,
            fold: false,
            add: true,
            cursor: false,
            title: "+ Add role override…".into(),
            scope: String::new(),
            value: String::new(),
            source: String::new(),
            meta: String::new(),
        },
    );
    if let Some(cursor) = rows.iter_mut().find(|row| !row.header && !row.add) {
        cursor.cursor = true;
    }
    rows
}

struct AgentLine {
    index: usize,
    name: String,
    mode: String,
    detail: String,
}

impl AgentLine {
    fn from_agent(agent: Agent, index: usize, world: &World) -> Self {
        let mode = world
            .global
            .agent_modes
            .get(&agent)
            .copied()
            .unwrap_or(AuthMode::Sync);
        let registry = match world.accounts.default_for(agent.provider()) {
            Some(account) => format!("★ {}", account.title()),
            None => match world.accounts.discovered_current(agent.provider()) {
                Some(account) => format!("discovered · {}", account.source.safe_detail()),
                None => "no account registered".into(),
            },
        };
        let detail = if mode == AuthMode::Ignore {
            "no credentials handed to the container".into()
        } else {
            registry
        };
        Self {
            index,
            name: agent.label().to_owned(),
            mode: auth_mode_label(mode).to_owned(),
            detail,
        }
    }
}

/// One trust row with its pending state resolved.
struct TrustItem {
    index: usize,
    source: String,
    kind: &'static str,
    trusted: bool,
    roles: usize,
    changed: bool,
    selected: bool,
}

/// Reusable global-settings screen composition (live Trust tab).
pub struct SettingsScreen;

impl SettingsScreen {
    /// Tab name for a 1-based settings tab (1 General … 5 Trust).
    pub fn tab_name(tab: usize) -> &'static str {
        TAB_NAMES[tab.saturating_sub(1).min(4)]
    }

    /// Header crumb for a 1-based settings tab (tag `crumb`).
    pub fn crumb(tab: usize) -> String {
        format!("Settings › global › {}", Self::tab_name(tab))
    }

    /// Footer hints for a 1-based settings tab and focus region (tag
    /// `hints`: tab strip, Cancel/Save… buttons, or the per-tab body).
    pub fn hints(tab: usize, focus: SettingsFocus) -> HintLayer {
        fn hint(key: &'static str, label: &'static str, priority: u8) -> Hint {
            Hint {
                key: HintKey::Label(key),
                label,
                priority,
            }
        }
        let tail = [
            hint("[ ]", "Switch tab", 70),
            hint("Ctrl+S", "Save", 60),
            hint("Esc", "Back", 50),
        ];
        let mut hints = match focus {
            SettingsFocus::Tabs => vec![
                hint("← →", "Tab", 100),
                hint("1–5", "Jump", 90),
                hint("Enter", "Body", 80),
            ],
            SettingsFocus::Buttons => vec![hint("← →", "Choose", 100), hint("Enter", "Run", 90)],
            SettingsFocus::Body => match tab {
                2 => vec![
                    hint("Enter", "Edit…", 100),
                    hint("r", "Read-only", 95),
                    hint("i", "Isolation", 90),
                    hint("o", "Open source", 85),
                    hint("d", "Remove", 80),
                    hint("s", "Scope…", 75),
                    hint("a", "Add mount…", 72),
                ],
                3 => vec![
                    hint("Enter", "Edit", 100),
                    hint("m", "Show", 95),
                    hint("p", "1Password…", 90),
                    hint("s", "Scope…", 85),
                    hint("d", "Remove…", 80),
                    hint("a", "Add…", 75),
                ],
                4 => vec![
                    hint("Space", "Cycle mode", 100),
                    hint("d", "Reset to sync", 90),
                    hint("c", "Manage accounts", 80),
                ],
                5 => vec![
                    hint("Space", "Toggle trust", 100),
                    hint("o", "Open source", 90),
                ],
                _ => vec![hint("Space", "Toggle", 100), hint("↑↓", "Move", 90)],
            },
        };
        hints.extend(tail);
        HintLayer {
            hints,
            badge: None,
            status: None,
            centered: true,
        }
    }

    /// Draw the live Trust tab: tab strip, role-source list, notes, and the
    /// Cancel/Save… actions (tag `render` + `render_trust`).
    pub fn draw_trust(
        ui: &mut Ui<'_>,
        settings: &SettingsState,
        world: &jackin_preview_sim::world::World,
        list_focused: bool,
    ) {
        let full = ui.full();
        let original = &world.global.trust;
        let dirty = settings.trust_dirty(original);
        Self::draw_tabs(ui, 4, dirty);

        // 2. Title and trust counts.
        let title_style = resolve_style(ui, Role::Fg(FgStep::Secondary), true);
        ui.paint_str(Rect::new(4, 6, 12, 1), "Role sources", title_style);
        let effective: Vec<bool> = (0..original.len())
            .map(|i| settings.trust_effective(original, i))
            .collect();
        let trusted = effective.iter().filter(|t| **t).count();
        let meta = format!(
            "{} · {}",
            plural(trusted, "trusted", "trusted"),
            plural(original.len() - trusted, "untrusted", "untrusted")
        );
        let faint = resolve_style(ui, Role::Fg(FgStep::Faint), false);
        let meta_w = width(&meta) as u16;
        let meta_x = full
            .width
            .saturating_sub(2)
            .saturating_sub(meta_w)
            .saturating_sub(2);
        ui.paint_str(Rect::new(meta_x, 6, meta_w, 1), &meta, faint);

        // 3. Role-source rows (stock list; per-cell styles mirror the tag).
        let list_width = full.width.saturating_sub(4);
        let src_w = (list_width.saturating_sub(30).clamp(16, 40)) as usize;
        let items: Vec<TrustItem> = original
            .iter()
            .enumerate()
            .map(|(index, row)| TrustItem {
                index,
                source: pad_cell(&row.source, src_w as u16),
                kind: row.kind,
                trusted: effective[index],
                roles: row.roles,
                changed: original
                    .get(index)
                    .is_some_and(|o| o.trusted != effective[index]),
                selected: index == settings.trust_cursor,
            })
            .collect();
        let mut list_state = ListState::default();
        list_state.set_cursor(settings.trust_cursor, ItemKey::index(settings.trust_cursor));
        list_state.choose(Some(ItemKey::index(settings.trust_cursor)));
        let mut trust_list = List::new(TRUST)
            .blank_marker(Role::Fg(FgStep::Secondary))
            .key(|item: &TrustItem| ItemKey::index(item.index))
            .row(move |item: &TrustItem, row| {
                let mut cols = row.columns_with_gap(
                    &[
                        Track::Fixed(1),
                        Track::Fixed(1),
                        Track::Fixed(src_w as u16),
                        Track::Fixed(2),
                        Track::Fixed(5),
                        Track::Fixed(1),
                        Track::Fixed(15),
                        Track::Flex(1),
                    ],
                    0,
                );
                let warn = StylePatch::new().set_fg(Role::Warning);
                cols.cell(0)
                    .patch(&warn)
                    .text(if item.changed { "•" } else { " " });
                if item.selected {
                    let bold = StylePatch::new().add(Modifier::BOLD);
                    cols.cell(2).patch(&bold).text(&item.source);
                } else {
                    cols.cell(2).text(&item.source);
                }
                let muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                cols.cell(4).patch(&muted).text(&pad_cell(item.kind, 5));
                let mark = if item.trusted {
                    "[✓] trusted"
                } else {
                    "[ ] untrusted"
                };
                if item.trusted {
                    cols.cell(6).text(mark);
                } else {
                    cols.cell(6).patch(&warn).text(mark);
                }
                let roles = plural(item.roles, "role", "roles");
                let kind_x = 9 + src_w as u16;
                let row_right = 2 + list_width;
                if kind_x + 22 + (width(&roles) as u16) < row_right {
                    let faint_patch = StylePatch::new().set_fg(Role::Fg(FgStep::Faint));
                    cols.cell(7).patch(&faint_patch).text(&roles);
                }
            });
        if !list_focused {
            trust_list = trust_list.focused(false);
        }
        trust_list.draw(
            ui,
            Rect::new(2, 7, list_width, (items.len() as u16).max(1)),
            &list_state,
            &items,
        );

        // 4. Untrusted-source notes.
        let note_y = 7 + items.len() as u16 + 1;
        if note_y + 1 < full.height.saturating_sub(2) {
            let note_w = full.width.saturating_sub(8);
            ui.paint_str(
                Rect::new(4, note_y, note_w, 1),
                &truncate(
                    "An untrusted source blocks + Load role until it is trusted here or",
                    note_w,
                ),
                faint,
            );
            ui.paint_str(
                Rect::new(4, note_y + 1, note_w, 1),
                &truncate("in the trust dialog that the load opens.", note_w),
                faint,
            );
        }

        Self::draw_actions(ui, dirty);
    }

    /// Draw every settings tab. `save_preview` covers Trust with the save dialog.
    pub fn draw(
        ui: &mut Ui<'_>,
        tab: usize,
        settings: &SettingsState,
        world: &World,
        save_preview: bool,
    ) {
        if save_preview {
            Self::draw_trust(ui, settings, world, false);
            let full = ui.full();
            ui.dim_layer(
                backdrop_area(
                    full,
                    Backdrop::Dim {
                        exclude_footer: true,
                    },
                ),
                2,
            );
            if ui.theme().capability.color == ColorLevel::Ansi16 {
                Self::draw_save_ansi16(ui);
            }
            Self::draw_save_preview(ui, settings, world);
            return;
        }
        match tab {
            2 => Self::draw_mounts(ui, world),
            3 => Self::draw_env(ui, world),
            4 => Self::draw_agents(ui, world),
            5 => Self::draw_trust(ui, settings, world, true),
            _ => Self::draw_general(ui, world),
        }
    }

    /// Footer hints while the save dialog is open.
    pub fn preview_hints() -> HintLayer {
        fn hint(key: &'static str, label: &'static str, priority: u8) -> Hint {
            Hint {
                key: HintKey::Label(key),
                label,
                priority,
            }
        }
        HintLayer {
            hints: vec![
                hint("← →", "Choose", 100),
                hint("Enter", "Confirm", 90),
                hint("Esc", "Cancel", 80),
            ],
            badge: None,
            status: None,
            centered: true,
        }
    }

    fn draw_tabs(ui: &mut Ui<'_>, active: usize, dirty_on_active: bool) {
        let full = ui.full();
        let tabs_all = [0usize, 1, 2, 3, 4];
        let mut tab_state = TabsState::default();
        let active = active.min(4);
        tab_state.set_active(active, ItemKey::num(active as u64));
        Tabs::new(TABS)
            .key(|tab: &usize| ItemKey::num(*tab as u64))
            .row(move |tab: &usize, row| {
                let name = TAB_NAMES[*tab];
                if dirty_on_active && *tab == active {
                    row.label(&format!("{name} •"));
                } else {
                    row.label(name);
                }
            })
            .draw(
                ui,
                Rect::new(2, 3, full.width.saturating_sub(4), 2),
                &tab_state,
                &tabs_all,
            );
    }

    fn draw_actions(ui: &mut Ui<'_>, save_enabled: bool) {
        let full = ui.full();
        let y = full.height.saturating_sub(3);
        // Both labels occupy six columns; the button frame adds two.
        let cancel_w = 8u16;
        let save_w = 8u16;
        let right = full.width.saturating_sub(4);
        let save_x = right.saturating_sub(save_w);
        let cancel_x = save_x.saturating_sub(3 + cancel_w);
        Button::new(CANCEL, "Cancel")
            .variant(Variant::GHOST)
            .draw(ui, Rect::new(cancel_x, y, cancel_w, 1));
        let mut save = Button::new(SAVE, "Save…");
        // An enabled Save is secondary on the overlay. The save-preview
        // backdrop then steps that foreground to faint, which is the
        // dimmed footer button in the frozen frame.
        let level = ui.theme().capability.color;
        // Truecolor dims a secondary label to faint. ANSI-16 and mono dim a
        // solid overlay fill (fg == bg) into the frozen gray or black block.
        let enabled = match level {
            ColorLevel::Ansi16 | ColorLevel::Mono => StylePatch::new()
                .set_fg(Role::Surface(Surface::Overlay))
                .set_bg(Role::Surface(Surface::Overlay)),
            _ => StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)),
        };
        if !save_enabled {
            save = save.disabled(true);
        } else {
            save = save.patch(&enabled);
        }
        save.draw(ui, Rect::new(save_x, y, save_w, 1));
    }

    /// ANSI-16 save-preview footer Save… is a solid dark-gray block.
    fn draw_save_ansi16(ui: &mut Ui<'_>) {
        let full = ui.full();
        let y = full.height.saturating_sub(3);
        let save_w = 8u16;
        let save_x = full.width.saturating_sub(4).saturating_sub(save_w);
        let solid = StylePatch::new()
            .set_fg(Role::Fg(FgStep::Faint))
            .set_bg(Role::Fg(FgStep::Faint));
        Button::new(SAVE, "Save…")
            .patch(&solid)
            .draw(ui, Rect::new(save_x, y, save_w, 1));
    }

    fn draw_general(ui: &mut Ui<'_>, world: &World) {
        Self::draw_tabs(ui, 0, false);
        let secondary = resolve_style(ui, Role::Fg(FgStep::Secondary), true);
        let faint = resolve_style(ui, Role::Fg(FgStep::Faint), false);
        let muted = resolve_style(ui, Role::Fg(FgStep::Muted), false);
        let full = ui.full();
        let x = 4u16;
        ui.paint_str(Rect::new(x, 6, 12, 1), "Commits", secondary);
        Checkbox::new(COAUTHOR, "Add Co-authored-by trailer")
            .checked(world.global.coauthor_trailer)
            .draw(ui, Rect::new(x, 7, 36, 1));
        Checkbox::new(DCO, "Sign off commits (DCO)")
            .checked(world.global.dco_signoff)
            .draw(ui, Rect::new(x, 8, 32, 1));
        ui.paint_str(
            Rect::new(x, 10, full.width.saturating_sub(x + 2), 1),
            "Two independent flags; both apply to every Workspace.",
            faint,
        );
        ui.paint_str(Rect::new(x, 12, 12, 1), "Trailers", secondary);
        let trailer = "Co-authored-by: <agent> <noreply@…>   ·   Signed-off-by: Alexey Zhokhov <alexey@chainargos.com>";
        let trailer_w = full.width.saturating_sub(8);
        ui.paint_str(
            Rect::new(x, 13, trailer_w, 1),
            &truncate(trailer, trailer_w),
            muted,
        );
        Self::draw_actions(ui, false);
    }

    fn draw_mounts(ui: &mut Ui<'_>, world: &World) {
        Self::draw_tabs(ui, 1, false);
        let full = ui.full();
        let list_w = full.width.saturating_sub(4);
        let widths = mount_widths(list_w.saturating_sub(7));
        let muted = resolve_style(ui, Role::Fg(FgStep::Muted), false);
        let faint = resolve_style(ui, Role::Fg(FgStep::Faint), false);
        let headers = [
            "Destination",
            "Scope",
            "Mode",
            "Isolation",
            "Kind",
            "Source",
        ];
        let mut x = 8u16;
        for (i, header) in headers.iter().enumerate() {
            let cw = widths[i];
            if cw == 0 {
                continue;
            }
            ui.paint_str(Rect::new(x, 6, cw, 1), &truncate(header, cw), muted);
            x = x.saturating_add(cw).saturating_add(2);
        }
        let hidden = widths.iter().filter(|w| **w == 0).count();
        if hidden > 0 {
            let tag = format!("{hidden}›");
            let tag_w = width(&tag) as u16;
            ui.paint_str(
                Rect::new(2 + list_w.saturating_sub(tag_w), 6, tag_w, 1),
                &tag,
                faint,
            );
        }
        let mut rows: Vec<MountRow> = world
            .global
            .mounts
            .iter()
            .enumerate()
            .map(|(index, mount)| {
                let mut row = MountRow::from_mount(mount, world);
                row.index = index;
                row
            })
            .collect();
        let mut add = MountRow::add();
        add.index = rows.len();
        rows.push(add);
        let mut state = ListState::default();
        state.set_cursor(0, ItemKey::index(0));
        state.choose(Some(ItemKey::index(0)));
        let (tracks, cells) = content_tracks(&widths);
        List::new(MOUNTS_BODY)
            .blank_marker(Role::Fg(FgStep::Secondary))
            .key(|row: &MountRow| ItemKey::index(row.index))
            .row(move |row: &MountRow, ui_row| {
                if row.add {
                    let live = ui_row
                        .flags()
                        .intersects(StateFlags::FOCUSED | StateFlags::HOVERED);
                    let fg = if live {
                        Role::Fg(FgStep::Primary)
                    } else {
                        Role::Fg(FgStep::Secondary)
                    };
                    let patch = StylePatch::new().set_fg(fg);
                    let mut cols = ui_row.columns_with_gap(&[Track::Fixed(3), Track::Flex(1)], 0);
                    cols.cell(1).patch(&patch).text(&row.cells[0]);
                    return;
                }
                let selected = ui_row.flags().contains(StateFlags::SELECTED);
                let mut cols = ui_row.columns_with_gap(&tracks, 0);
                let muted_patch = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                let secondary = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                let bold = StylePatch::new().add(Modifier::BOLD);
                for (i, cell) in row.cells.iter().enumerate() {
                    let Some(slot) = cells.get(i).copied() else {
                        continue;
                    };
                    if slot == usize::MAX {
                        continue;
                    }
                    let text = pad_cell(cell, widths[i]);
                    if i == 0 {
                        if selected {
                            cols.cell(slot).patch(&bold).text(&text);
                        } else {
                            cols.cell(slot).text(&text);
                        }
                    } else if i == 1 || i >= 4 {
                        cols.cell(slot).patch(&muted_patch).text(&text);
                    } else {
                        cols.cell(slot).patch(&secondary).text(&text);
                    }
                }
            })
            .draw(
                ui,
                Rect::new(2, 7, list_w, rows.len() as u16),
                &state,
                &rows,
            );
        Self::draw_actions(ui, false);
    }

    fn draw_env(ui: &mut Ui<'_>, world: &World) {
        Self::draw_tabs(ui, 2, false);
        let full = ui.full();
        let body_w = full.width.saturating_sub(4);
        // Tag `config.rs`: a body at least 150 columns keeps a 56-column
        // inspector and a 2-column gap, so the list loses 58 columns.
        let wide = body_w >= 150;
        let list_w = if wide {
            body_w.saturating_sub(58)
        } else {
            body_w
        };
        let widths = env_widths(list_w.saturating_sub(7));
        let rows = env_rows(world, &widths);
        let cursor = rows.iter().position(|row| row.cursor).unwrap_or(0);
        let mut state = ListState::default();
        state.set_cursor(cursor, ItemKey::index(cursor));
        state.choose(Some(ItemKey::index(cursor)));
        let (tracks, cells) = content_tracks(&widths);
        List::new(ENV_BODY)
            .blank_marker(Role::Fg(FgStep::Secondary))
            .bare_item(&env_section_header)
            .key(|row: &EnvLine| ItemKey::index(row.index))
            .row(move |row: &EnvLine, ui_row| {
                if row.header {
                    let bold = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Secondary))
                        .add(Modifier::BOLD);
                    let faint = StylePatch::new().set_fg(Role::Fg(FgStep::Faint));
                    let secondary = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    if !row.meta.is_empty() {
                        ui_row.meta_patched(&row.meta, &faint);
                    }
                    let mut cols = ui_row
                        .columns_with_gap(&[Track::Fixed(2), Track::Fixed(2), Track::Flex(1)], 0);
                    if row.fold {
                        cols.cell(1).patch(&secondary).text("▾");
                    }
                    cols.cell(2).patch(&bold).text(&row.title);
                    return;
                }
                if row.add {
                    let live = ui_row
                        .flags()
                        .intersects(StateFlags::FOCUSED | StateFlags::HOVERED);
                    let fg = if live {
                        Role::Fg(FgStep::Primary)
                    } else {
                        Role::Fg(FgStep::Secondary)
                    };
                    let patch = StylePatch::new().set_fg(fg);
                    let mut cols = ui_row.columns_with_gap(&[Track::Fixed(3), Track::Flex(1)], 0);
                    cols.cell(1).patch(&patch).text(&row.title);
                    return;
                }
                let selected = ui_row.flags().contains(StateFlags::SELECTED);
                let mut cols = ui_row.columns_with_gap(&tracks, 0);
                let muted_patch = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                let secondary = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                let bold = StylePatch::new().add(Modifier::BOLD);
                let values = [&row.title, &row.scope, &row.value, &row.source];
                for (i, text) in values.iter().enumerate() {
                    let Some(slot) = cells.get(i).copied() else {
                        continue;
                    };
                    if slot == usize::MAX || widths[i] == 0 {
                        continue;
                    }
                    let padded = pad_cell(text, widths[i]);
                    if i == 0 {
                        if selected {
                            cols.cell(slot).patch(&bold).text(&padded);
                        } else {
                            cols.cell(slot).text(&padded);
                        }
                    } else if i == 2 {
                        cols.cell(slot).patch(&secondary).text(&padded);
                    } else {
                        cols.cell(slot).patch(&muted_patch).text(&padded);
                    }
                }
            })
            .draw(
                ui,
                Rect::new(2, 6, list_w, rows.len() as u16),
                &state,
                &rows,
            );
        if wide {
            if let Some(row) = rows.iter().find(|row| row.cursor) {
                Self::draw_env_card(ui, world, row);
            }
        }
        Self::draw_actions(ui, false);
    }

    fn draw_env_card(ui: &mut Ui<'_>, world: &World, row: &EnvLine) {
        let full = ui.full();
        let body_h = full.height.saturating_sub(10);
        let area = Rect::new(full.width.saturating_sub(58), 6, 56, body_h.min(24));
        let found = env_var(world, &row.scope, &row.title);
        let pending = found
            .map(|var| var.value.source_label())
            .unwrap_or("removed");
        let title = format!("Variable · {}", row.title);
        let mut lines = vec![
            ("Pending", pending.to_owned(), Role::Fg(FgStep::Primary)),
            ("Original", pending.to_owned(), Role::Fg(FgStep::Muted)),
        ];
        if let Some(EnvValue::OnePassword(reference)) = found.map(|var| &var.value) {
            lines.push(("Reference", reference.canonical(), Role::Fg(FgStep::Muted)));
            lines.push((
                "Vault",
                format!("{} · {}", reference.account, reference.vault_name),
                Role::Fg(FgStep::Muted),
            ));
        }
        lines.push((
            "Resolution",
            "resolved at launch · never stored in the Construct image".to_owned(),
            Role::Fg(FgStep::Faint),
        ));
        let props: Vec<PropsRow<'_>> = lines
            .iter()
            .enumerate()
            .map(|(index, (key, value, tone))| {
                PropsRow::new(ItemKey::index(index), *key, value).tone(*tone)
            })
            .collect();
        Panel::new(ENV_CARD)
            .title(&title)
            .meta(&row.scope)
            .draw(ui, area, |ui, inner| {
                Props::rich(&props).label_column(14).draw(ui, inner);
            });
    }

    fn draw_agents(ui: &mut Ui<'_>, world: &World) {
        Self::draw_tabs(ui, 3, false);
        let full = ui.full();
        let secondary = resolve_style(ui, Role::Fg(FgStep::Secondary), true);
        let faint = resolve_style(ui, Role::Fg(FgStep::Faint), false);
        let muted = resolve_style(ui, Role::Fg(FgStep::Muted), false);
        ui.paint_str(Rect::new(4, 6, 22, 1), "Agent runtime mode", secondary);
        let meta = "accounts are registered in Accounts (c)";
        if full.width > 50 {
            let meta_w = width(meta) as u16;
            let meta_x = full
                .width
                .saturating_sub(2)
                .saturating_sub(meta_w)
                .saturating_sub(2);
            ui.paint_str(Rect::new(meta_x, 6, meta_w, 1), meta, faint);
        }
        ui.paint_str(
            Rect::new(7, 7, full.width.saturating_sub(9), 1),
            &format!("{:<14}{:<13}{}", "Agent", "Mode", "Registry default"),
            muted,
        );
        let rows: Vec<AgentLine> = Agent::ALL
            .iter()
            .enumerate()
            .map(|(index, agent)| AgentLine::from_agent(*agent, index, world))
            .collect();
        let mut state = ListState::default();
        state.set_cursor(0, ItemKey::index(0));
        state.choose(Some(ItemKey::index(0)));
        let row_w = full.width.saturating_sub(4);
        let detail_w = row_w.saturating_sub(33);
        List::new(AGENTS_BODY)
            .blank_marker(Role::Fg(FgStep::Secondary))
            .key(|row: &AgentLine| ItemKey::index(row.index))
            .row(move |row: &AgentLine, ui_row| {
                let selected = ui_row.flags().contains(StateFlags::SELECTED);
                let mut cols = ui_row.columns_with_gap(
                    &[
                        Track::Fixed(1),
                        Track::Fixed(1),
                        Track::Fixed(13),
                        Track::Fixed(1),
                        Track::Fixed(12),
                        Track::Fixed(1),
                        Track::Flex(1),
                    ],
                    0,
                );
                let secondary = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                let muted_patch = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                let warn = StylePatch::new().set_fg(Role::Warning);
                let bold = StylePatch::new().add(Modifier::BOLD);
                cols.cell(0).patch(&warn).text(" ");
                if selected {
                    cols.cell(2).patch(&bold).text(&pad_cell(&row.name, 13));
                } else {
                    cols.cell(2).text(&pad_cell(&row.name, 13));
                }
                cols.cell(4)
                    .patch(&secondary)
                    .text(&pad_cell(&row.mode, 12));
                cols.cell(6)
                    .patch(&muted_patch)
                    .text(&truncate(&row.detail, detail_w));
            })
            .draw(ui, Rect::new(2, 8, row_w, rows.len() as u16), &state, &rows);
        let note_y = 8 + rows.len() as u16 + 1;
        let note_w = full.width.saturating_sub(8);
        if note_y + 1 < full.height.saturating_sub(4) {
            ui.paint_str(
                Rect::new(4, note_y, note_w, 1),
                &truncate(
                    "sync mirrors the host login · api key and oauth token take material from the registry account",
                    note_w,
                ),
                faint,
            );
            ui.paint_str(
                Rect::new(4, note_y + 1, note_w, 1),
                &truncate(
                    "ignore starts the agent without credentials and removes it from session pickers",
                    note_w,
                ),
                faint,
            );
        }
        Self::draw_actions(ui, false);
    }

    fn draw_save_preview(ui: &mut Ui<'_>, settings: &SettingsState, world: &World) {
        let original = &world.global.trust;
        let n = settings.trust_change_count(original);
        let scope = format!(
            "global config · {}",
            world.tilde(&format!("{}/.jackin/config.toml", world.home))
        );
        let changes = plural(n, "change", "changes");
        let trust = original
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                let pending = settings.trust_effective(original, index);
                (pending != row.trusted).then(|| {
                    format!(
                        "{} → {}",
                        row.source,
                        if pending { "trusted" } else { "untrusted" }
                    )
                })
            })
            .collect::<Vec<_>>()
            .join(" · ");
        let code = original
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                let pending = settings.trust_effective(original, index);
                (pending != row.trusted)
                    .then(|| format!("~ trust {} {} → {}", row.source, row.trusted, pending))
            })
            .collect::<Vec<_>>();
        let code_refs: Vec<&str> = code.iter().map(String::as_str).collect();
        let actions = [
            Action::new(ActionKey::CANCEL, "Cancel"),
            Action::new(ActionKey::CONFIRM, "Save"),
        ];
        let dialog = Dialog::new(PREVIEW)
            .title("Save settings")
            .actions(&actions)
            .cancel(ActionKey::CANCEL)
            .primary(ActionKey::CONFIRM)
            .width(66)
            .body_rows(3)
            .code(&code_refs);
        let primary = Role::Fg(FgStep::Primary);
        let rows = [
            PropsRow::new(ItemKey::index(0), "Scope", &scope).tone(primary),
            PropsRow::new(ItemKey::index(1), "Changes", &changes).tone(primary),
            PropsRow::new(ItemKey::index(2), "Trust", &trust).tone(primary),
        ];
        let state = DialogState::default();
        let frame = resolve_anchor(
            ui.full(),
            Anchor::Screen(ScreenAlign::Center),
            LayerSize::Fixed(
                dialog.measured_width(ui.design()),
                dialog.measured_height(ui.design()),
            ),
        );
        dialog.draw(ui, frame, &state, |ui, body| {
            Props::rich(&rows).draw(ui, body);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_save_keeps_dirty_draft_and_retry_commits() {
        let mut state = SettingsState::default();
        state.begin_draft();
        assert!(state.attempt_save(true));
        assert!(!state.is_clean());
        assert!(state.save_error.is_some());

        assert!(!state.attempt_save(true));
        assert!(state.is_clean());
        assert!(state.save_error.is_none());
    }

    #[test]
    fn a_new_edit_clears_old_error_and_restarts_attempts() {
        let mut state = SettingsState::default();
        state.begin_draft();
        assert!(state.attempt_save(true));
        state.begin_draft();
        assert_eq!(state.save_attempts, 0);
        assert!(state.save_error.is_none());
        assert!(!state.is_clean());
    }

    fn trust_fixture() -> Vec<TrustRow> {
        vec![
            TrustRow {
                source: "github.com/chainargos/roles".into(),
                kind: "git",
                trusted: true,
                roles: 4,
            },
            TrustRow {
                source: "github.com/acme-labs/roles-experimental".into(),
                kind: "git",
                trusted: false,
                roles: 1,
            },
        ]
    }

    #[test]
    fn save_preview_flow_yields_two_row_targeted_changes() {
        let original = trust_fixture();
        let mut state = SettingsState::default();
        assert_eq!(state.trust_change_count(&original), 0);
        assert!(state.is_clean());

        let (source, trusted) = state.toggle_trust(&original).unwrap();
        assert_eq!(source, "github.com/chainargos/roles");
        assert!(!trusted);
        assert_eq!(state.trust_change_count(&original), 1);
        assert!(!state.is_clean());

        state.move_trust_cursor(1, original.len());
        let (source, trusted) = state.toggle_trust(&original).unwrap();
        assert_eq!(source, "github.com/acme-labs/roles-experimental");
        assert!(trusted);
        assert_eq!(state.trust_change_count(&original), 2);
        assert!(!state.is_clean());
    }

    #[test]
    fn toggling_back_to_world_value_drops_the_override() {
        let original = trust_fixture();
        let mut state = SettingsState::default();
        state.toggle_trust(&original);
        assert_eq!(state.trust_change_count(&original), 1);
        state.toggle_trust(&original);
        assert_eq!(state.trust_change_count(&original), 0);
        assert!(state.trust_overrides.is_empty());
        assert!(state.is_clean());
    }

    #[test]
    fn trust_cursor_clamps_and_toggle_out_of_range_is_none() {
        let original = trust_fixture();
        let mut state = SettingsState::default();
        state.move_trust_cursor(-5, original.len());
        assert_eq!(state.trust_cursor, 0);
        state.move_trust_cursor(99, original.len());
        assert_eq!(state.trust_cursor, original.len() - 1);
        state.trust_cursor = 99;
        assert!(state.toggle_trust(&original).is_none());
        assert_eq!(state.trust_change_count(&original), 0);
    }

    #[test]
    fn out_of_range_overrides_never_count() {
        let original = trust_fixture();
        let mut state = SettingsState::default();
        state.trust_overrides.insert(99, true);
        assert_eq!(state.trust_change_count(&original), 0);
        assert!(!state.trust_dirty(&original));
    }

    #[test]
    fn padded_cells_fill_the_column_and_agent_modes_use_baseline_words() {
        assert_eq!(pad_cell("sync", 12), "sync        ");
        assert_eq!(pad_cell("Claude Code", 13), "Claude Code  ");
        assert_eq!(auth_mode_label(AuthMode::ApiKey), "api key");
        assert_eq!(auth_mode_label(AuthMode::Ignore), "ignore");
        assert_eq!(
            Agent::ALL.map(Agent::label),
            [
                "Claude Code",
                "Codex",
                "Amp",
                "Kimi Code",
                "OpenCode",
                "Grok Build",
            ]
        );
    }

    #[test]
    fn crumbs_name_every_tab() {
        assert_eq!(SettingsScreen::crumb(1), "Settings › global › General");
        assert_eq!(SettingsScreen::crumb(2), "Settings › global › Mounts");
        assert_eq!(SettingsScreen::crumb(3), "Settings › global › Environments");
        assert_eq!(SettingsScreen::crumb(4), "Settings › global › Agents");
        assert_eq!(SettingsScreen::crumb(5), "Settings › global › Trust");
    }

    #[test]
    fn hints_follow_focus_and_tab() {
        let tabs = SettingsScreen::hints(5, SettingsFocus::Tabs);
        let labels: Vec<&str> = tabs.hints.iter().map(|h| h.label).collect();
        assert!(labels.contains(&"Jump"));
        assert!(labels.contains(&"Body"));
        assert!(!labels.contains(&"Toggle trust"));

        let trust = SettingsScreen::hints(5, SettingsFocus::Body);
        let labels: Vec<&str> = trust.hints.iter().map(|h| h.label).collect();
        assert!(labels.contains(&"Toggle trust"));
        assert!(labels.contains(&"Open source"));
        assert!(labels.contains(&"Switch tab"));

        let agents = SettingsScreen::hints(4, SettingsFocus::Body);
        let labels: Vec<&str> = agents.hints.iter().map(|h| h.label).collect();
        assert!(labels.contains(&"Cycle mode"));

        let buttons = SettingsScreen::hints(5, SettingsFocus::Buttons);
        let labels: Vec<&str> = buttons.hints.iter().map(|h| h.label).collect();
        assert!(labels.contains(&"Choose"));
        assert!(labels.contains(&"Run"));
    }
}
