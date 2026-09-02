//! Shared Mounts / Environments / Auth tabs for the Workspace Editor and the
//! Global Settings. Both screens keep an original and a pending `Doc`; rows
//! carry `+ • −` change slots, child modals edit one row at a time and the
//! save preview is built from the diff.

use std::collections::{BTreeMap, HashSet};

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{fit, truncate, width, wrap};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::choice::{Checkbox, RadioGroup};
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::select::Select;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use super::modals::{BrowserResult, ChoiceDialog, FieldKindW, FieldValue, FileBrowser, FormDialog, FormField, FormValues, OpFlow};
use super::{Cx, Modal, ModalResult, ModalTag};
use crate::domain::account::CredentialSource;
use crate::domain::agent::{Agent, AuthMode};
use crate::domain::fixtures::{PrecedenceLevel, resolve_account};
use crate::domain::onepassword::OpReference;
use crate::domain::workspace::{AuthEntry, AuthSource, EnvValue, EnvVar, Isolation, Mount, MountKind, MountScope, MountSource, RoleName, Workspace, env_key_error, mask};
use crate::sim::world::{GlobalConfig, World};

/// Which configuration document the tabs edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Workspace,
    Global,
}

impl Scope {
    fn label(self) -> &'static str {
        match self {
            Scope::Workspace => "workspace",
            Scope::Global => "global",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Mounts,
    Environments,
    Auth,
}

/// The parts of a Workspace or the global config that the shared tabs edit.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Doc {
    pub mounts: Vec<Mount>,
    pub env: Vec<EnvVar>,
    pub role_env: BTreeMap<RoleName, Vec<EnvVar>>,
    pub auth: Vec<AuthEntry>,
    pub role_auth: BTreeMap<RoleName, Vec<AuthEntry>>,
}

impl Doc {
    pub fn from_workspace(w: &Workspace) -> Self {
        Self {
            mounts: w.mounts.clone(),
            env: w.env.clone(),
            role_env: w.role_env.clone(),
            auth: w.auth.clone(),
            role_auth: w.role_auth.clone(),
        }
    }

    pub fn apply_to_workspace(&self, w: &mut Workspace) {
        w.mounts = self.mounts.clone();
        w.env = self.env.clone();
        w.role_env = self.role_env.clone();
        w.auth = self.auth.clone();
        w.role_auth = self.role_auth.clone();
    }

    pub fn from_global(g: &GlobalConfig) -> Self {
        Self {
            mounts: g.mounts.clone(),
            env: g.env.clone(),
            role_env: g.role_env.clone(),
            auth: g.auth.clone(),
            role_auth: g.role_auth.clone(),
        }
    }

    pub fn apply_to_global(&self, g: &mut GlobalConfig) {
        g.mounts = self.mounts.clone();
        g.env = self.env.clone();
        g.role_env = self.role_env.clone();
        g.auth = self.auth.clone();
        g.role_auth = self.role_auth.clone();
    }

    fn env_of(&self, role: Option<&str>) -> &[EnvVar] {
        match role {
            None => &self.env,
            Some(r) => self.role_env.get(r).map(Vec::as_slice).unwrap_or(&[]),
        }
    }

    fn env_of_mut(&mut self, role: Option<&str>) -> &mut Vec<EnvVar> {
        match role {
            None => &mut self.env,
            Some(r) => self.role_env.entry(r.to_owned()).or_default(),
        }
    }

    fn auth_of(&self, role: Option<&str>) -> &[AuthEntry] {
        match role {
            None => &self.auth,
            Some(r) => self.role_auth.get(r).map(Vec::as_slice).unwrap_or(&[]),
        }
    }

    fn auth_of_mut(&mut self, role: Option<&str>) -> &mut Vec<AuthEntry> {
        match role {
            None => &mut self.auth,
            Some(r) => self.role_auth.entry(r.to_owned()).or_default(),
        }
    }

    fn roles(&self) -> Vec<RoleName> {
        let mut v: Vec<RoleName> = self.role_env.keys().chain(self.role_auth.keys()).cloned().collect();
        v.sort();
        v.dedup();
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    None,
    Added,
    Modified,
    Removed,
}

impl Change {
    fn glyph(self) -> &'static str {
        match self {
            Change::None => " ",
            Change::Added => "+",
            Change::Modified => "•",
            Change::Removed => "−",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKey {
    Mount(String),
    RemovedMount(String),
    AddMount,
    Section(Option<RoleName>),
    Env(Option<RoleName>, String),
    AddEnv(Option<RoleName>),
    Auth(Option<RoleName>, Agent),
    AddAuth(Option<RoleName>),
    Blocker,
}

#[derive(Debug, Clone)]
struct Row {
    key: RowKey,
    change: Change,
    cells: Vec<(String, Tone)>,
    /// Trailing `!` with a message (blocker or validation).
    problem: Option<String>,
    faint: bool,
    header: bool,
    folded: Option<bool>,
    meta: String,
}

#[derive(Default)]
struct ListState {
    rows: Vec<Row>,
    cursor: usize,
    scroll: ScrollState,
    area: Rect,
}

/// What a child modal edits, so its result lands on the right row.
#[derive(Debug, Clone)]
enum Editing {
    Mount { original: Option<String> },
    Env { role: Option<RoleName>, original: Option<String> },
    Auth { role: Option<RoleName>, original: Option<Agent> },
}

pub struct ConfigTabs {
    pub scope: Scope,
    pub original: Doc,
    pub pending: Doc,
    removed_mounts: Vec<Mount>,
    mounts: ListState,
    envs: ListState,
    auth: ListState,
    folded: HashSet<Option<RoleName>>,
    unmasked: HashSet<(Option<RoleName>, String)>,
    editing: Option<Editing>,
    op_ref: Option<OpReference>,
    /// Roles the scope selector offers (registry + roles already used).
    roles: Vec<RoleName>,
    ids: Ids,
    pub status_hint: Option<String>,
}

#[derive(Clone, Copy)]
struct Ids {
    mounts: WidgetId,
    envs: WidgetId,
    auth: WidgetId,
    form: WidgetId,
}

impl ConfigTabs {
    pub fn new(scope: Scope, original: Doc, pending: Doc, base: WidgetId, roles: Vec<RoleName>) -> Self {
        let mut roles = roles;
        for r in original.roles().into_iter().chain(pending.roles()) {
            if !roles.contains(&r) {
                roles.push(r);
            }
        }
        Self {
            scope,
            original,
            pending,
            removed_mounts: vec![],
            mounts: ListState::default(),
            envs: ListState::default(),
            auth: ListState::default(),
            folded: HashSet::new(),
            unmasked: HashSet::new(),
            editing: None,
            op_ref: None,
            roles,
            ids: Ids {
                mounts: base.sub("mounts"),
                envs: base.sub("envs"),
                auth: base.sub("auth"),
                form: base.sub("form"),
            },
            status_hint: None,
        }
    }

    pub fn list_id(&self, tab: Tab) -> WidgetId {
        match tab {
            Tab::Mounts => self.ids.mounts,
            Tab::Environments => self.ids.envs,
            Tab::Auth => self.ids.auth,
        }
    }

    fn state(&mut self, tab: Tab) -> &mut ListState {
        match tab {
            Tab::Mounts => &mut self.mounts,
            Tab::Environments => &mut self.envs,
            Tab::Auth => &mut self.auth,
        }
    }

    fn state_ref(&self, tab: Tab) -> &ListState {
        match tab {
            Tab::Mounts => &self.mounts,
            Tab::Environments => &self.envs,
            Tab::Auth => &self.auth,
        }
    }

    // ------------------------------------------------------------ dirty

    pub fn change_count(&self) -> usize {
        let a = &self.pending;
        let b = &self.original;
        diff_len(&a.mounts, &b.mounts) + diff_len(&a.env, &b.env) + diff_len(&a.auth, &b.auth) + usize::from(a.role_env != b.role_env) + usize::from(a.role_auth != b.role_auth)
    }

    pub fn tab_dirty(&self, tab: Tab) -> bool {
        match tab {
            Tab::Mounts => self.pending.mounts != self.original.mounts,
            Tab::Environments => self.pending.env != self.original.env || self.pending.role_env != self.original.role_env,
            Tab::Auth => self.pending.auth != self.original.auth || self.pending.role_auth != self.original.role_auth,
        }
    }

    pub fn tab_error(&self, tab: Tab) -> bool {
        match tab {
            Tab::Mounts => self.pending.mounts.iter().any(|m| m.drift.is_some() || m.running_isolated && self.original.mounts.iter().all(|o| o != m)),
            Tab::Environments => self.pending.env.iter().chain(self.pending.role_env.values().flatten()).any(|e| env_key_error(&e.key).is_some()),
            Tab::Auth => false,
        }
    }

    /// Blockers that stop a save until cleaned up.
    pub fn blockers(&self) -> Vec<String> {
        let mut v = vec![];
        for m in &self.pending.mounts {
            if let Some(d) = &m.drift {
                v.push(format!("{} · source drift: {d}", m.destination));
            }
            if m.running_isolated && self.original.mounts.iter().all(|o| o != m) {
                v.push(format!("{} · a running instance holds isolated state for this mount; eject it first", m.destination));
            }
        }
        v
    }

    /// Clears every blocker (simulated cleanup) so the save can proceed.
    pub fn clean_up(&mut self) {
        for m in &mut self.pending.mounts {
            m.drift = None;
            m.running_isolated = false;
        }
    }

    /// `+ − ~` preview lines for the save facts dialog.
    pub fn diff_lines(&self) -> Vec<String> {
        let mut v = vec![];
        for m in &self.pending.mounts {
            match self.original.mounts.iter().find(|o| o.destination == m.destination) {
                None => v.push(format!("+ mount {}  {} · {} · {}", m.destination, m.mode_label(), m.isolation.label().to_lowercase(), m.source_label())),
                Some(o) if o != m => v.push(format!("~ mount {}  {} · {} → {} · {}", m.destination, o.mode_label(), o.isolation.label().to_lowercase(), m.mode_label(), m.isolation.label().to_lowercase())),
                _ => {}
            }
        }
        for o in &self.original.mounts {
            if !self.pending.mounts.iter().any(|m| m.destination == o.destination) {
                v.push(format!("− mount {}", o.destination));
            }
        }
        let scopes: Vec<Option<RoleName>> = std::iter::once(None).chain(self.roles.iter().cloned().map(Some)).collect();
        for s in &scopes {
            let a = self.pending.env_of(s.as_deref());
            let b = self.original.env_of(s.as_deref());
            let sc = s.as_deref().map(|r| format!(" (role {r})")).unwrap_or_default();
            for e in a {
                match b.iter().find(|o| o.key == e.key) {
                    None => v.push(format!("+ env {}{sc}  {}", e.key, e.value.source_label())),
                    Some(o) if o != e => v.push(format!("~ env {}{sc}  {} → {}", e.key, o.value.source_label(), e.value.source_label())),
                    _ => {}
                }
            }
            for o in b {
                if !a.iter().any(|e| e.key == o.key) {
                    v.push(format!("− env {}{sc}", o.key));
                }
            }
            let a = self.pending.auth_of(s.as_deref());
            let b = self.original.auth_of(s.as_deref());
            for e in a {
                match b.iter().find(|o| o.agent == e.agent) {
                    None => v.push(format!("+ auth {}{sc}  {} · {}", e.agent.label(), mode_label(e.mode), source_kind(&e.source))),
                    Some(o) if o != e => v.push(format!("~ auth {}{sc}  {} · {} → {} · {}", e.agent.label(), mode_label(o.mode), source_kind(&o.source), mode_label(e.mode), source_kind(&e.source))),
                    _ => {}
                }
            }
            for o in b {
                if !a.iter().any(|e| e.agent == o.agent) {
                    v.push(format!("− auth {}{sc}", o.agent.label()));
                }
            }
        }
        v
    }

    pub fn summary_facts(&self) -> Vec<Prop> {
        let mut v = vec![];
        let (ma, mm, mr) = counts(&self.pending.mounts, &self.original.mounts, |m| m.destination.clone());
        if ma + mm + mr > 0 {
            v.push(Prop::new("Mounts", format!("{ma} added · {mm} modified · {mr} removed")));
        }
        let mut ea = 0;
        let mut em = 0;
        let mut er = 0;
        let mut aa = 0;
        let mut am = 0;
        let mut ar = 0;
        let scopes: Vec<Option<RoleName>> = std::iter::once(None).chain(self.roles.iter().cloned().map(Some)).collect();
        for s in &scopes {
            let (a, m, r) = counts(self.pending.env_of(s.as_deref()), self.original.env_of(s.as_deref()), |e| e.key.clone());
            ea += a;
            em += m;
            er += r;
            let (a, m, r) = counts(self.pending.auth_of(s.as_deref()), self.original.auth_of(s.as_deref()), |e| e.agent.label().to_owned());
            aa += a;
            am += m;
            ar += r;
        }
        if ea + em + er > 0 {
            v.push(Prop::new("Environments", format!("{ea} added · {em} modified · {er} removed")));
        }
        if aa + am + ar > 0 {
            v.push(Prop::new("Auth", format!("{aa} added · {am} modified · {ar} removed")));
        }
        v
    }

    // ------------------------------------------------------------- rows

    fn build_rows(&mut self, tab: Tab, w: &World) {
        let rows = match tab {
            Tab::Mounts => self.mount_rows(w),
            Tab::Environments => self.env_rows(),
            Tab::Auth => self.auth_rows(w),
        };
        let st = self.state(tab);
        st.rows = rows;
        if st.cursor >= st.rows.len() {
            st.cursor = st.rows.len().saturating_sub(1);
        }
        // never rest on a header without a fold
        st.scroll.set_content(st.rows.len());
    }

    fn mount_rows(&self, w: &World) -> Vec<Row> {
        let mut rows = vec![];
        for m in &self.pending.mounts {
            let change = match self.original.mounts.iter().find(|o| o.destination == m.destination) {
                None => Change::Added,
                Some(o) if o != m => Change::Modified,
                _ => Change::None,
            };
            let problem = if let Some(d) = &m.drift {
                Some(format!("{} · source drift: {d}. Re-choose the source or remove the mount before saving.", m.destination))
            } else if m.running_isolated && change != Change::None {
                Some(format!("{} · a running instance holds isolated state for this mount; eject it first.", m.destination))
            } else {
                None
            };
            let mut cells = vec![
                (m.destination.clone(), Tone::Normal),
                (m.mode_label().into(), Tone::Secondary),
                (m.isolation.label().to_lowercase(), Tone::Secondary),
                (if matches!(m.source, MountSource::Git(_)) { "git".into() } else { "host".into() }, Tone::Muted),
                (w.tilde(m.source_label()), Tone::Muted),
            ];
            if self.scope == Scope::Global {
                cells.insert(1, (m.scope.label(), Tone::Muted));
            }
            rows.push(Row {
                key: RowKey::Mount(m.destination.clone()),
                change,
                cells,
                problem,
                faint: false,
                header: false,
                folded: None,
                meta: String::new(),
            });
        }
        for m in &self.removed_mounts {
            let mut cells = vec![
                (m.destination.clone(), Tone::Faint),
                (m.mode_label().into(), Tone::Faint),
                (m.isolation.label().to_lowercase(), Tone::Faint),
                (if matches!(m.source, MountSource::Git(_)) { "git".into() } else { "host".into() }, Tone::Faint),
                (w.tilde(m.source_label()), Tone::Faint),
            ];
            if self.scope == Scope::Global {
                cells.insert(1, (m.scope.label(), Tone::Faint));
            }
            rows.push(Row {
                key: RowKey::RemovedMount(m.destination.clone()),
                change: Change::Removed,
                cells,
                problem: None,
                faint: true,
                header: false,
                folded: None,
                meta: "u restores".into(),
            });
        }
        rows.push(Row {
            key: RowKey::AddMount,
            change: Change::None,
            cells: vec![("+ Add mount".into(), Tone::Secondary)],
            problem: None,
            faint: false,
            header: false,
            folded: None,
            meta: String::new(),
        });
        rows
    }

    fn env_rows(&self) -> Vec<Row> {
        let mut rows = vec![];
        let scopes: Vec<Option<RoleName>> = std::iter::once(None).chain(self.roles.iter().cloned().map(Some)).collect();
        for s in scopes {
            let vars = self.pending.env_of(s.as_deref());
            let orig = self.original.env_of(s.as_deref());
            let folded = self.folded.contains(&s);
            let in_registry = s.as_deref().is_none_or(|r| self.roles.iter().any(|x| x == r));
            let title = match &s {
                None => self.scope.label().to_owned().replace("workspace", "Workspace").replace("global", "Global"),
                Some(r) => format!("Role: {r}"),
            };
            let mut meta = format!("{} vars", vars.len());
            if !in_registry {
                meta = format!("not in registry · {meta}");
            }
            rows.push(Row {
                key: RowKey::Section(s.clone()),
                change: Change::None,
                cells: vec![(title, Tone::Normal)],
                problem: None,
                faint: false,
                header: true,
                folded: if s.is_some() { Some(folded) } else { None },
                meta,
            });
            if s.is_some() && folded {
                continue;
            }
            for e in vars {
                let change = match orig.iter().find(|o| o.key == e.key) {
                    None => Change::Added,
                    Some(o) if o != e => Change::Modified,
                    _ => Change::None,
                };
                let shown = self.unmasked.contains(&(s.clone(), e.key.clone()));
                let (value, source) = match &e.value {
                    EnvValue::Plain(v) => (
                        if shown { v.clone() } else { mask(v) },
                        if shown { "plain · shown".to_owned() } else { "plain".to_owned() },
                    ),
                    EnvValue::OnePassword(r) => (mask_len(16), format!("[op] {}", r.display_path())),
                    EnvValue::HostEnv(name) => (format!("${name}"), "host env".into()),
                };
                let mut cells = vec![(e.key.clone(), Tone::Normal), (value, Tone::Secondary), (source, Tone::Muted)];
                if self.scope == Scope::Global {
                    cells.insert(1, (s.as_deref().map(|r| format!("role {r}")).unwrap_or("global".into()), Tone::Muted));
                }
                rows.push(Row {
                    key: RowKey::Env(s.clone(), e.key.clone()),
                    change,
                    cells,
                    problem: env_key_error(&e.key),
                    faint: false,
                    header: false,
                    folded: None,
                    meta: String::new(),
                });
            }
            for o in orig {
                if !vars.iter().any(|e| e.key == o.key) {
                    let mut cells = vec![(o.key.clone(), Tone::Faint), (mask_len(12), Tone::Faint), (o.value.source_label().into(), Tone::Faint)];
                    if self.scope == Scope::Global {
                        cells.insert(1, (s.as_deref().map(|r| format!("role {r}")).unwrap_or("global".into()), Tone::Faint));
                    }
                    rows.push(Row {
                        key: RowKey::Env(s.clone(), o.key.clone()),
                        change: Change::Removed,
                        cells,
                        problem: None,
                        faint: true,
                        header: false,
                        folded: None,
                        meta: "u restores".into(),
                    });
                }
            }
            rows.push(Row {
                key: RowKey::AddEnv(s.clone()),
                change: Change::None,
                cells: vec![(
                    match &s {
                        None => "+ Add environment variable".into(),
                        Some(r) => format!("+ Add {r} environment variable"),
                    },
                    Tone::Secondary,
                )],
                problem: None,
                faint: false,
                header: false,
                folded: None,
                meta: String::new(),
            });
        }
        rows
    }

    fn auth_rows(&self, w: &World) -> Vec<Row> {
        let mut rows = vec![];
        let scopes: Vec<Option<RoleName>> = std::iter::once(None).chain(self.roles.iter().cloned().map(Some)).collect();
        for s in scopes {
            let entries = self.pending.auth_of(s.as_deref());
            let orig = self.original.auth_of(s.as_deref());
            let folded = self.folded.contains(&s);
            let title = match &s {
                None => match self.scope {
                    Scope::Workspace => "Workspace".to_owned(),
                    Scope::Global => "Global auth per agent runtime".to_owned(),
                },
                Some(r) => format!("Role: {r}"),
            };
            let meta = if s.is_none() {
                match self.scope {
                    Scope::Workspace => "inherits global".into(),
                    Scope::Global => "defaults from Accounts (c)".into(),
                }
            } else if entries.is_empty() {
                format!("inherits {}", self.scope.label())
            } else {
                format!("{} override{}", entries.len(), if entries.len() == 1 { "" } else { "s" })
            };
            rows.push(Row {
                key: RowKey::Section(s.clone()),
                change: Change::None,
                cells: vec![(title, Tone::Normal)],
                problem: None,
                faint: false,
                header: true,
                folded: if s.is_some() { Some(folded) } else { None },
                meta,
            });
            if s.is_some() && folded {
                continue;
            }
            let agents: Vec<Agent> = if s.is_none() {
                Agent::ALL.to_vec()
            } else {
                entries.iter().map(|e| e.agent).collect()
            };
            for agent in agents {
                let entry = entries.iter().find(|e| e.agent == agent);
                let change = match (entry, orig.iter().find(|o| o.agent == agent)) {
                    (Some(e), Some(o)) if e != o => Change::Modified,
                    (Some(_), None) => Change::Added,
                    _ => Change::None,
                };
                let (mode, source, detail, tone) = match entry {
                    Some(e) => (mode_label(e.mode).to_owned(), source_kind(&e.source).to_owned(), source_detail(&e.source, w), Tone::Normal),
                    None => {
                        // inherited: the global entry, else the built-in default
                        let g = w.global.auth.iter().find(|e| e.agent == agent);
                        match (self.scope, g) {
                            (Scope::Workspace, Some(e)) => (mode_label(e.mode).to_owned(), "global".to_owned(), format!("{} · {}", source_kind(&e.source), source_detail(&e.source, w)), Tone::Muted),
                            _ => ("inherit".to_owned(), "".to_owned(), builtin_default(agent, w), Tone::Muted),
                        }
                    }
                };
                let mut cells = vec![(agent.label().to_owned(), tone), (mode, tone), (source, tone), (detail, if tone == Tone::Normal { Tone::Muted } else { Tone::Faint })];
                if self.scope == Scope::Global && s.is_some() {
                    cells.insert(1, (format!("role {}", s.as_deref().unwrap_or("")), Tone::Muted));
                } else if self.scope == Scope::Global {
                    cells.insert(1, ("global".into(), Tone::Muted));
                }
                rows.push(Row {
                    key: RowKey::Auth(s.clone(), agent),
                    change,
                    cells,
                    problem: None,
                    faint: entry.is_none(),
                    header: false,
                    folded: None,
                    meta: String::new(),
                });
            }
            for o in orig {
                if !entries.iter().any(|e| e.agent == o.agent) && s.is_some() {
                    rows.push(Row {
                        key: RowKey::Auth(s.clone(), o.agent),
                        change: Change::Removed,
                        cells: vec![(o.agent.label().to_owned(), Tone::Faint), (mode_label(o.mode).to_owned(), Tone::Faint), (source_kind(&o.source).to_owned(), Tone::Faint), (String::new(), Tone::Faint)],
                        problem: None,
                        faint: true,
                        header: false,
                        folded: None,
                        meta: "u restores".into(),
                    });
                }
            }
            rows.push(Row {
                key: RowKey::AddAuth(s.clone()),
                change: Change::None,
                cells: vec![(
                    match &s {
                        None => "+ Add provider override".into(),
                        Some(r) => format!("+ Add {r} override"),
                    },
                    Tone::Secondary,
                )],
                problem: None,
                faint: false,
                header: false,
                folded: None,
                meta: String::new(),
            });
        }
        rows
    }

    // ----------------------------------------------------------- render

    /// Renders the tab body; returns the footer status for the focused row.
    pub fn render(&mut self, tab: Tab, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) -> Option<String> {
        self.build_rows(tab, w);
        let t = ctx.theme;
        let bg = t.canvas;
        let id = self.list_id(tab);
        let focused = ctx.interaction.focused(id);
        let scope = self.scope;
        let wide = area.width >= 150 && tab != Tab::Mounts;
        let (list_area, card_area) = if wide {
            (Rect::new(area.x, area.y, area.width - 58, area.height), Some(Rect::new(area.right() - 56, area.y, 56, area.height.min(24))))
        } else {
            (area, None)
        };
        // header row for tables
        let header: Vec<&str> = match (tab, scope) {
            (Tab::Mounts, Scope::Workspace) => vec!["Destination", "Mode", "Isolation", "Kind", "Source"],
            (Tab::Mounts, Scope::Global) => vec!["Destination", "Scope", "Mode", "Isolation", "Kind", "Source"],
            (Tab::Environments, Scope::Workspace) => vec!["Key", "Value", "Source"],
            (Tab::Environments, Scope::Global) => vec!["Key", "Scope", "Value", "Source"],
            (Tab::Auth, Scope::Workspace) => vec!["Agent", "Mode", "Source", "Account · folder · reference"],
            (Tab::Auth, Scope::Global) => vec!["Agent", "Scope", "Mode", "Source", "Account · folder · reference"],
        };
        let widths = column_widths(tab, scope, list_area.width.saturating_sub(7));
        let hidden = widths.iter().filter(|w| **w == 0).count();
        let mut y = list_area.y;
        if tab == Tab::Mounts {
            let mut x = list_area.x + 7;
            for (i, h) in header.iter().enumerate() {
                if widths[i] == 0 {
                    continue;
                }
                buf.set_string(x, y, truncate(h, widths[i] as usize), t.muted().bg(bg));
                x += widths[i] + 2;
            }
            if hidden > 0 {
                let tag = format!("{hidden}›");
                buf.set_string(list_area.right().saturating_sub(width(&tag) as u16), y, &tag, t.faint().bg(bg));
            }
            y += 1;
        }
        let blocker_rows = {
            let st = self.state_ref(tab);
            st.rows.iter().filter_map(|r| r.problem.clone()).collect::<Vec<_>>()
        };
        let reserve = if blocker_rows.is_empty() { 0 } else { 1 + blocker_rows.len().min(2) as u16 * 2 };
        let body = Rect::new(list_area.x, y, list_area.width, list_area.bottom().saturating_sub(y + reserve));
        let cursor = self.state_ref(tab).cursor;
        let st = self.state(tab);
        st.area = body;
        st.scroll.set_viewport(body.height as usize);
        st.scroll.ensure_visible(cursor);
        ctx.control(id, body, false);
        ctx.scrollable(id, body);
        let has_sb = st.scroll.overflows();
        let row_w = body.width.saturating_sub(u16::from(has_sb));
        let range = st.scroll.visible_range();
        let rows = st.rows.clone();
        let mut status = None;
        for (k, i) in range.enumerate() {
            let y = body.y + k as u16;
            let row = &rows[i];
            let rid = id.child(i);
            let mut s = ctx.state(rid);
            s.focused = focused && i == cursor;
            s.selected = i == cursor;
            let style = if row.header { Style::new().bg(bg) } else { t.row(s, bg) };
            let rect = Rect::new(body.x, y, row_w, 1);
            fill(buf, rect, style);
            if row.header {
                let mut x = rect.x + 2;
                if let Some(f) = row.folded {
                    buf.set_string(x, y, if f { "▸" } else { "▾" }, t.secondary().bg(bg));
                    ctx.clickable(rid.sub("fold"), Rect::new(x, y, 2, 1));
                }
                x += 2;
                let title = &row.cells[0].0;
                buf.set_string(x, y, title, if s.selected { t.primary().bg(bg).add_modifier(Modifier::BOLD) } else { t.secondary().bg(bg).add_modifier(Modifier::BOLD) });
                if s.selected {
                    buf.set_string(rect.x, y, "▎", t.gutter(s, bg, false));
                }
                let mw = width(&row.meta) as u16;
                if mw > 0 && rect.width > mw + width(title) as u16 + 8 {
                    let tone = if row.meta.starts_with("not in registry") { t.warning } else { t.text_faint };
                    buf.set_string(rect.right().saturating_sub(mw + 1), y, &row.meta, Style::new().fg(tone).bg(bg));
                }
                ctx.clickable(rid, rect);
                continue;
            }
            buf.set_string(rect.x, y, "▎", t.gutter(s, style.bg.unwrap_or(bg), false));
            let sel_glyph = if s.selected { "›" } else { " " };
            buf.set_string(rect.x + 1, y, sel_glyph, style.fg(if s.focused { t.accent } else { t.text_secondary }));
            let change_style = match row.change {
                Change::Added => style.fg(t.text_primary),
                Change::Modified => style.fg(t.warning),
                Change::Removed => style.fg(t.text_faint),
                Change::None => style,
            };
            buf.set_string(rect.x + 3, y, row.change.glyph(), change_style);
            let mut x = rect.x + 6;
            let is_add = matches!(row.key, RowKey::AddMount | RowKey::AddEnv(_) | RowKey::AddAuth(_));
            if is_add {
                buf.set_string(x, y, &row.cells[0].0, style.fg(if s.focused { t.text_primary } else { t.text_secondary }));
            } else {
                for (ci, (text, tone)) in row.cells.iter().enumerate() {
                    let cw = widths.get(ci).copied().unwrap_or(0);
                    if cw == 0 {
                        continue;
                    }
                    let mut cs = style.fg(if row.faint { t.text_faint } else { t.tone(*tone) });
                    if s.selected && ci == 0 && !row.faint {
                        cs = cs.fg(t.text_primary).add_modifier(Modifier::BOLD);
                    }
                    if row.change == Change::Removed {
                        cs = cs.add_modifier(Modifier::CROSSED_OUT);
                    }
                    buf.set_string(x, y, fit(&truncate(text, cw as usize), cw as usize), cs);
                    x += cw + 2;
                }
            }
            if row.problem.is_some() {
                buf.set_string(rect.right().saturating_sub(2), y, "!", style.fg(t.error));
            } else if !row.meta.is_empty() && s.selected {
                let mw = width(&row.meta) as u16;
                buf.set_string(rect.right().saturating_sub(mw + 1), y, &row.meta, style.fg(t.text_faint));
            }
            ctx.clickable(rid, rect);
            if s.selected {
                status = self.row_status(tab, &row.key, hidden > 0, w);
            }
        }
        let st = self.state(tab);
        if has_sb {
            scrollbar::render_vertical(Rect::new(body.right() - 1, body.y, 1, body.height), buf, ctx, id, &st.scroll, focused);
        }
        if reserve > 0 {
            let mut y = body.bottom() + 1;
            for p in blocker_rows.iter().take(2) {
                for (i, line) in wrap(p, list_area.width.saturating_sub(5) as usize).into_iter().take(2).enumerate() {
                    if y >= list_area.bottom() {
                        break;
                    }
                    buf.set_string(list_area.x + 3, y, if i == 0 { "!" } else { " " }, Style::new().fg(t.error).bg(bg));
                    buf.set_string(list_area.x + 5, y, &line, Style::new().fg(t.error).bg(bg));
                    y += 1;
                }
            }
        }
        if let Some(card) = card_area {
            self.render_inspector(tab, card, buf, ctx, w);
        }
        status
    }

    fn row_status(&self, tab: Tab, key: &RowKey, hidden: bool, w: &World) -> Option<String> {
        match key {
            RowKey::Mount(d) => {
                let m = self.pending.mounts.iter().find(|m| &m.destination == d)?;
                let o = self.original.mounts.iter().find(|m| &m.destination == d);
                match o {
                    Some(o) if o != m => Some(format!("was {} · {}{}", o.mode_label(), o.isolation.label().to_lowercase(), if o.source != m.source { format!(" · {}", w.tilde(o.source_label())) } else { String::new() })),
                    None => Some("new mount".into()),
                    _ if hidden => Some(format!("source {}", w.tilde(m.source_label()))),
                    _ => None,
                }
            }
            RowKey::Env(role, k) => {
                let o = self.original.env_of(role.as_deref()).iter().find(|e| &e.key == k);
                let p = self.pending.env_of(role.as_deref()).iter().find(|e| &e.key == k);
                match (o, p) {
                    (Some(o), Some(p)) if o != p => Some(format!("was {}", o.value.source_label())),
                    (None, Some(_)) => Some("new variable".into()),
                    (Some(_), None) => Some("removed · u restores".into()),
                    _ => None,
                }
            }
            RowKey::Auth(role, agent) => {
                let o = self.original.auth_of(role.as_deref()).iter().find(|e| e.agent == *agent);
                let p = self.pending.auth_of(role.as_deref()).iter().find(|e| e.agent == *agent);
                match (o, p) {
                    (Some(o), Some(p)) if o != p => Some(format!("was {} · {} · {}", mode_label(o.mode), source_kind(&o.source), source_detail(&o.source, w))),
                    (None, Some(_)) => Some("new override".into()),
                    (Some(_), None) => Some("removed · u restores".into()),
                    (None, None) if tab == Tab::Auth => Some(format!("inherited · {}", builtin_default(*agent, w))),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn render_inspector(&mut self, tab: Tab, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let st = self.state_ref(tab);
        let Some(row) = st.rows.get(st.cursor) else { return };
        let (title, meta, lines): (String, String, Vec<(String, String, Tone)>) = match &row.key {
            RowKey::Auth(role, agent) => {
                let scope = role.as_deref().map(|r| format!("role {r} scope")).unwrap_or(format!("{} scope", self.scope.label()));
                let mut lines = vec![];
                let p = self.pending.auth_of(role.as_deref()).iter().find(|e| e.agent == *agent);
                let o = self.original.auth_of(role.as_deref()).iter().find(|e| e.agent == *agent);
                lines.push(("Pending".into(), p.map(|e| format!("{} · {}", mode_label(e.mode), source_detail(&e.source, w))).unwrap_or("inherited".into()), Tone::Normal));
                lines.push(("Original".into(), o.map(|e| format!("{} · {}", mode_label(e.mode), source_detail(&e.source, w))).unwrap_or("inherited".into()), Tone::Muted));
                {
                    let provider = agent.provider();
                    let ws = w.workspaces.first();
                    let r = resolve_account(provider, ws, role.as_deref(), None, &w.accounts);
                    lines.push(("Precedence".into(), String::new(), Tone::Muted));
                    for lvl in [PrecedenceLevel::Session, PrecedenceLevel::Role, PrecedenceLevel::Workspace, PrecedenceLevel::ProviderDefault, PrecedenceLevel::Discovered] {
                        let here = r.level == lvl;
                        let value = match lvl {
                            PrecedenceLevel::Session => "none".to_owned(),
                            PrecedenceLevel::Workspace => ws.and_then(|x| x.account_overrides.get(&provider)).and_then(|id| w.accounts.get(id)).map(|a| a.display_name.clone()).unwrap_or("none".into()),
                            PrecedenceLevel::Role => role.as_deref().and_then(|rn| ws.and_then(|x| x.role_account_overrides.get(&(rn.to_owned(), provider)))).and_then(|id| w.accounts.get(id)).map(|a| a.display_name.clone()).unwrap_or("none".into()),
                            PrecedenceLevel::ProviderDefault => w.accounts.default_for(provider).map(|a| a.display_name.clone()).unwrap_or("none".into()),
                            PrecedenceLevel::Discovered => w.accounts.discovered_current(provider).map(|a| a.source.safe_detail()).unwrap_or("none".into()),
                            PrecedenceLevel::None => "none".into(),
                        };
                        lines.push((format!("  {} {}", if here { "›" } else { " " }, lvl.label()), value, if here { Tone::Normal } else { Tone::Muted }));
                    }
                    if let Some(a) = r.account.as_ref().and_then(|id| w.accounts.get(id)) {
                        lines.push(("Health".into(), format!("{} · {}", a.status_word(), a.last_refresh_secs.map(|s| w.clock.ago(s)).unwrap_or("never".into())), Tone::Normal));
                        lines.push(("Credential".into(), format!("{} · {}", a.source.origin_label(), a.source.safe_detail()), Tone::Normal));
                    }
                }
                lines.push(("Modes".into(), agent.auth_modes().iter().map(|m| mode_label(*m)).collect::<Vec<_>>().join(" · "), Tone::Muted));
                (format!("Effective auth · {}", agent.label()), scope, lines)
            }
            RowKey::Env(role, key) => {
                let p = self.pending.env_of(role.as_deref()).iter().find(|e| &e.key == key);
                let o = self.original.env_of(role.as_deref()).iter().find(|e| &e.key == key);
                let mut lines = vec![];
                lines.push(("Pending".into(), p.map(|e| e.value.source_label().to_owned()).unwrap_or("removed".into()), Tone::Normal));
                lines.push(("Original".into(), o.map(|e| e.value.source_label().to_owned()).unwrap_or("new".into()), Tone::Muted));
                if let Some(EnvValue::OnePassword(r)) = p.map(|e| &e.value) {
                    lines.push(("Reference".into(), r.canonical(), Tone::Muted));
                    lines.push(("Vault".into(), format!("{} · {}", r.account, r.vault_name), Tone::Muted));
                }
                lines.push(("Resolution".into(), "resolved at launch · never stored in the Construct image".into(), Tone::Faint));
                (format!("Variable · {key}"), role.as_deref().map(|r| format!("role {r}")).unwrap_or(self.scope.label().into()), lines)
            }
            _ => return,
        };
        let inner = Panel::card(Some(&title)).meta(&meta).render(area, buf, t);
        let bg = t.surface;
        let lw = 14u16;
        let mut y = inner.y;
        for (k, v, tone) in lines {
            if y >= inner.bottom() {
                break;
            }
            buf.set_string(inner.x, y, truncate(&k, lw as usize), t.muted().bg(bg));
            buf.set_string(inner.x + lw, y, truncate(&v, inner.width.saturating_sub(lw) as usize), Style::new().fg(t.tone(tone)).bg(bg));
            y += 1;
        }
    }

    // ------------------------------------------------------------- keys

    pub fn hints(&self, tab: Tab) -> Vec<Hint> {
        let st = self.state_ref(tab);
        let key = st.rows.get(st.cursor).map(|r| r.key.clone());
        let mut v = vec![];
        match (tab, key) {
            (Tab::Mounts, Some(RowKey::Mount(_))) => {
                v.push(hint("Enter", "Edit…"));
                v.push(hint("r", "Read-only"));
                v.push(hint("i", "Isolation"));
                v.push(hint("o", "Open source"));
                v.push(hint("d", "Remove"));
                if self.scope == Scope::Global {
                    v.push(hint("s", "Scope…"));
                }
                v.push(hint("a", "Add mount…"));
            }
            (Tab::Mounts, Some(RowKey::RemovedMount(_))) => {
                v.push(hint("u", "Undo remove"));
                v.push(hint("a", "Add mount…"));
            }
            (Tab::Mounts, _) => v.push(hint("Enter", "Add mount…")),
            (Tab::Environments, Some(RowKey::Env(..))) => {
                v.push(hint("Enter", "Edit"));
                v.push(hint("m", "Show"));
                v.push(hint("p", "1Password…"));
                v.push(hint("s", "Scope…"));
                v.push(hint("d", "Remove…"));
                v.push(hint("a", "Add…"));
            }
            (Tab::Environments, Some(RowKey::Section(Some(_)))) => {
                v.push(hint("← →", "Fold role"));
                v.push(hint("a", "Add…"));
            }
            (Tab::Environments, _) => {
                v.push(hint("Enter", "Add…"));
            }
            (Tab::Auth, Some(RowKey::Auth(..))) => {
                v.push(hint("Enter", "Edit auth…"));
                v.push(hint("Space", "Mode"));
                v.push(hint("d", if self.scope == Scope::Global { "Reset" } else { "Reset to inherited" }));
                v.push(hint("a", "Add override…"));
                if self.scope == Scope::Global {
                    v.push(hint("c", "Manage accounts"));
                }
            }
            (Tab::Auth, Some(RowKey::Section(Some(_)))) => {
                v.push(hint("← →", "Fold role"));
                v.push(hint("a", "Add override…"));
            }
            (Tab::Auth, _) => v.push(hint("Enter", "Add override…")),
        }
        v
    }

    fn move_cursor(&mut self, tab: Tab, delta: isize) {
        let st = self.state(tab);
        let n = st.rows.len();
        if n == 0 {
            return;
        }
        let mut i = st.cursor as isize;
        loop {
            i = (i + delta).clamp(0, n as isize - 1);
            let r = &st.rows[i as usize];
            if r.header && r.folded.is_none() {
                if i == 0 && delta < 0 || i == n as isize - 1 && delta > 0 {
                    // sit on the header when it is the boundary
                    break;
                }
                continue;
            }
            break;
        }
        st.cursor = i as usize;
        st.scroll.ensure_visible(st.cursor);
    }

    pub fn on_key(&mut self, tab: Tab, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        self.build_rows(tab, w);
        let st = self.state_ref(tab);
        let Some(row) = st.rows.get(st.cursor).cloned() else { return Outcome::Ignored };
        let vp = st.scroll.viewport_len.max(1) as isize;
        let n = st.rows.len() as isize;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_cursor(tab, -1);
                return Outcome::Changed;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_cursor(tab, 1);
                return Outcome::Changed;
            }
            KeyCode::PageUp => {
                self.move_cursor(tab, -vp);
                return Outcome::Changed;
            }
            KeyCode::PageDown => {
                self.move_cursor(tab, vp);
                return Outcome::Changed;
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.move_cursor(tab, -n);
                return Outcome::Changed;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.move_cursor(tab, n);
                return Outcome::Changed;
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                if let RowKey::Section(Some(r)) = &row.key {
                    let s = Some(r.clone());
                    if matches!(key.code, KeyCode::Left | KeyCode::Char('h')) {
                        self.folded.insert(s);
                    } else {
                        self.folded.remove(&s);
                    }
                    return Outcome::Changed;
                }
                return Outcome::Ignored;
            }
            KeyCode::Char('a') if key.plain() => {
                let role = match &row.key {
                    RowKey::Section(r) | RowKey::Env(r, _) | RowKey::AddEnv(r) | RowKey::Auth(r, _) | RowKey::AddAuth(r) => r.clone(),
                    _ => None,
                };
                match tab {
                    Tab::Mounts => self.open_mount_form(None, w, cx),
                    Tab::Environments => self.open_env_form(role, None, w, cx),
                    Tab::Auth => self.open_auth_form(role, None, w, cx),
                }
                return Outcome::Changed;
            }
            KeyCode::Char('u') if key.plain() => {
                return self.undo(tab, &row.key, w, cx);
            }
            _ => {}
        }
        match (&row.key, key.code) {
            (RowKey::AddMount, KeyCode::Enter) => self.open_mount_form(None, w, cx),
            (RowKey::AddEnv(r), KeyCode::Enter) => self.open_env_form(r.clone(), None, w, cx),
            (RowKey::AddAuth(r), KeyCode::Enter) => self.open_auth_form(r.clone(), None, w, cx),
            (RowKey::Section(Some(r)), KeyCode::Enter | KeyCode::Char(' ')) => {
                let s = Some(r.clone());
                if !self.folded.remove(&s) {
                    self.folded.insert(s);
                }
            }
            (RowKey::Mount(d), KeyCode::Enter | KeyCode::Char('e') | KeyCode::Char('n')) => self.open_mount_form(Some(d.clone()), w, cx),
            (RowKey::Mount(d), KeyCode::Char('r')) => {
                if let Some(m) = self.pending.mounts.iter_mut().find(|m| &m.destination == d) {
                    m.readonly = !m.readonly;
                    cx.status(format!("{} · {}", m.destination, if m.readonly { "read-only" } else { "read-write" }));
                }
            }
            (RowKey::Mount(d), KeyCode::Char('i')) => {
                if let Some(m) = self.pending.mounts.iter_mut().find(|m| &m.destination == d) {
                    if m.running_isolated {
                        cx.error(format!("Cannot change isolation: a running instance holds isolated state for {}", m.destination));
                    } else {
                        m.isolation = m.isolation.next();
                        cx.status(format!("{} · isolation {}", m.destination, m.isolation.label().to_lowercase()));
                    }
                }
            }
            (RowKey::Mount(d), KeyCode::Char(c @ ('1' | '2' | '3'))) => {
                if let Some(m) = self.pending.mounts.iter_mut().find(|m| &m.destination == d) {
                    m.isolation = match c {
                        '1' => Isolation::Shared,
                        '2' => Isolation::Worktree,
                        _ => Isolation::Clone,
                    };
                    cx.status(format!("{} · isolation {}", m.destination, m.isolation.label().to_lowercase()));
                }
            }
            (RowKey::Mount(d), KeyCode::Char('o')) => {
                if let Some(m) = self.pending.mounts.iter().find(|m| &m.destination == d) {
                    match &m.source {
                        MountSource::Git(u) => cx.status(format!("Opened https://{u} in the browser")),
                        MountSource::Host(p) if m.kind == MountKind::Repository => cx.status(format!("Opened {} on GitHub", w.tilde(p))),
                        _ => cx.error("no GitHub URL for this mount"),
                    }
                }
            }
            (RowKey::Mount(d), KeyCode::Char('d') | KeyCode::Delete) => {
                if let Some(i) = self.pending.mounts.iter().position(|m| &m.destination == d) {
                    let m = self.pending.mounts.remove(i);
                    if self.original.mounts.iter().any(|o| o.destination == m.destination) {
                        self.removed_mounts.push(m.clone());
                    }
                    cx.status(format!("Removed {} · u restores until save", m.destination));
                }
            }
            (RowKey::Mount(d), KeyCode::Char('s')) if self.scope == Scope::Global => {
                self.open_scope_picker(RowKey::Mount(d.clone()), cx);
            }
            (RowKey::RemovedMount(_), KeyCode::Enter) => return self.undo(tab, &row.key, w, cx),
            (RowKey::Env(r, k), KeyCode::Enter | KeyCode::Char('e')) => {
                if row.change == Change::Removed {
                    return self.undo(tab, &row.key, w, cx);
                }
                self.open_env_form(r.clone(), Some(k.clone()), w, cx);
            }
            (RowKey::Env(r, k), KeyCode::Char('m')) => {
                let id = (r.clone(), k.clone());
                let plain = self.pending.env_of(r.as_deref()).iter().any(|e| &e.key == k && matches!(e.value, EnvValue::Plain(_)));
                if !plain {
                    cx.status("Only plain values can be shown · references resolve at launch");
                } else if !self.unmasked.remove(&id) {
                    self.unmasked.insert(id);
                    cx.status(format!("{k} shown · m masks it again"));
                }
            }
            (RowKey::Env(r, k), KeyCode::Char('p')) => {
                self.editing = Some(Editing::Env {
                    role: r.clone(),
                    original: Some(k.clone()),
                });
                let flow = OpFlow::new(self.ids.form.sub("op"), &w.op, w.now_ms());
                cx.open(Modal::Op(flow), ModalTag::new("cfg.env.op"));
            }
            (RowKey::Env(r, k), KeyCode::Char('s')) => {
                self.open_scope_picker(RowKey::Env(r.clone(), k.clone()), cx);
            }
            (RowKey::Env(r, k), KeyCode::Char('d') | KeyCode::Delete) => {
                if row.change == Change::Removed {
                    return Outcome::Consumed;
                }
                let d = Dialog::destructive(self.ids.form.sub("envrm"), &format!("Delete environment variable {k}?"), &format!("The variable leaves the {} scope on save. Any 1Password reference stays in the vault.", r.as_deref().map(|x| format!("role {x}")).unwrap_or(self.scope.label().into())), "Delete");
                cx.open(Modal::Dialog(d), ModalTag::new("cfg.env.remove").key(&format!("{}\u{1}{}", r.clone().unwrap_or_default(), k)));
            }
            (RowKey::Auth(r, agent), KeyCode::Enter | KeyCode::Char('e')) => {
                if row.change == Change::Removed {
                    return self.undo(tab, &row.key, w, cx);
                }
                self.open_auth_form(r.clone(), Some(*agent), w, cx);
            }
            (RowKey::Auth(r, agent), KeyCode::Char(' ')) => {
                let modes = agent.auth_modes();
                let entries = self.pending.auth_of_mut(r.as_deref());
                match entries.iter_mut().find(|e| e.agent == *agent) {
                    Some(e) => {
                        let i = modes.iter().position(|m| *m == e.mode).unwrap_or(0);
                        e.mode = modes[(i + 1) % modes.len()];
                        if e.mode == AuthMode::Ignore {
                            e.source = AuthSource::None;
                        } else if e.source == AuthSource::None {
                            e.source = AuthSource::HostProfile;
                        }
                        cx.status(format!("{} · mode {}", agent.label(), mode_label(e.mode)));
                    }
                    None => {
                        entries.push(AuthEntry {
                            agent: *agent,
                            mode: modes[0],
                            source: AuthSource::HostProfile,
                        });
                        cx.status(format!("{} · override added · mode {}", agent.label(), mode_label(modes[0])));
                    }
                }
            }
            (RowKey::Auth(r, agent), KeyCode::Char('d') | KeyCode::Delete) => {
                let entries = self.pending.auth_of_mut(r.as_deref());
                let before = entries.len();
                entries.retain(|e| e.agent != *agent);
                if entries.len() < before {
                    cx.status(format!("{} · reset to inherited", agent.label()));
                } else {
                    cx.status(format!("{} already inherits", agent.label()));
                }
            }
            _ => return Outcome::Ignored,
        }
        Outcome::Changed
    }

    fn undo(&mut self, tab: Tab, key: &RowKey, _w: &World, cx: &mut Cx) -> Outcome {
        match (tab, key) {
            (Tab::Mounts, RowKey::RemovedMount(d)) => {
                if let Some(i) = self.removed_mounts.iter().position(|m| &m.destination == d) {
                    let m = self.removed_mounts.remove(i);
                    cx.status(format!("Restored {}", m.destination));
                    self.pending.mounts.push(m);
                }
                Outcome::Changed
            }
            (Tab::Mounts, RowKey::Mount(d)) => {
                if let Some(o) = self.original.mounts.iter().find(|m| &m.destination == d).cloned() {
                    if let Some(m) = self.pending.mounts.iter_mut().find(|m| &m.destination == d) {
                        *m = o;
                    }
                    cx.status(format!("Reverted {d}"));
                } else {
                    cx.status("Nothing to undo");
                }
                Outcome::Changed
            }
            (Tab::Environments, RowKey::Env(r, k)) => {
                let o = self.original.env_of(r.as_deref()).iter().find(|e| &e.key == k).cloned();
                let list = self.pending.env_of_mut(r.as_deref());
                match o {
                    Some(o) => {
                        if let Some(e) = list.iter_mut().find(|e| &e.key == k) {
                            *e = o;
                        } else {
                            list.push(o);
                        }
                        cx.status(format!("Restored {k}"));
                    }
                    None => cx.status("Nothing to undo"),
                }
                Outcome::Changed
            }
            (Tab::Auth, RowKey::Auth(r, a)) => {
                let o = self.original.auth_of(r.as_deref()).iter().find(|e| e.agent == *a).cloned();
                let list = self.pending.auth_of_mut(r.as_deref());
                match o {
                    Some(o) => {
                        if let Some(e) = list.iter_mut().find(|e| e.agent == *a) {
                            *e = o;
                        } else {
                            list.push(o);
                        }
                        cx.status(format!("Restored {}", a.label()));
                    }
                    None => {
                        list.retain(|e| e.agent != *a);
                        cx.status(format!("{} · back to inherited", a.label()));
                    }
                }
                Outcome::Changed
            }
            _ => {
                cx.status("Nothing to undo");
                Outcome::Consumed
            }
        }
    }

    pub fn on_click(&mut self, tab: Tab, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        self.build_rows(tab, w);
        let list = self.list_id(tab);
        let st = self.state_ref(tab);
        for i in st.scroll.visible_range() {
            if list.child(i).sub("fold") == id {
                if let RowKey::Section(Some(r)) = &st.rows[i].key {
                    let s = Some(r.clone());
                    if !self.folded.remove(&s) {
                        self.folded.insert(s);
                    }
                }
                self.state(tab).cursor = i;
                cx.focus.focus(list);
                return Outcome::Changed;
            }
            if list.child(i) == id {
                let same = st.cursor == i;
                self.state(tab).cursor = i;
                cx.focus.focus(list);
                if same {
                    let key = Key {
                        code: KeyCode::Enter,
                        mods: ratatui::crossterm::event::KeyModifiers::NONE,
                    };
                    return self.on_key(tab, &key, w, cx);
                }
                return Outcome::Changed;
            }
        }
        if id == scrollbar::id_for(list) {
            let st = self.state(tab);
            let track = Rect::new(st.area.right() - 1, st.area.y, 1, st.area.height);
            st.scroll.scroll_to(scrollbar::offset_for_click(track, pos, &st.scroll));
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    pub fn on_wheel(&mut self, tab: Tab, id: WidgetId, delta: i32) -> Outcome {
        let list = self.list_id(tab);
        if id == list || id == scrollbar::id_for(list) {
            self.state(tab).scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    // ------------------------------------------------------------ modals

    fn scope_options(&self) -> Vec<String> {
        let mut v = vec![self.scope.label().to_owned()];
        v.extend(self.roles.iter().map(|r| format!("role {r}")));
        v
    }

    fn open_scope_picker(&mut self, key: RowKey, cx: &mut Cx) {
        let opts = self.scope_options();
        let refs: Vec<&str> = opts.iter().map(String::as_str).collect();
        let (title, current) = match &key {
            RowKey::Mount(d) => (
                format!("Scope for {d}"),
                self.pending.mounts.iter().find(|m| &m.destination == d).map(|m| match &m.scope {
                    MountScope::Role(r) => 1 + self.roles.iter().position(|x| x == r).unwrap_or(0),
                    _ => 0,
                }),
            ),
            RowKey::Env(r, k) => (format!("Scope for {k}"), Some(r.as_deref().map(|r| 1 + self.roles.iter().position(|x| x == r).unwrap_or(0)).unwrap_or(0))),
            _ => return,
        };
        let d = ChoiceDialog::new(self.ids.form.sub("scope"), &title, "Scope", &refs, current.unwrap_or(0)).width(54);
        let k = match &key {
            RowKey::Mount(d) => format!("m\u{1}{d}"),
            RowKey::Env(r, k) => format!("e\u{1}{}\u{1}{k}", r.clone().unwrap_or_default()),
            _ => String::new(),
        };
        cx.open(Modal::Choice(d), ModalTag::new("cfg.scope").key(&k));
    }

    fn open_mount_form(&mut self, original: Option<String>, w: &World, cx: &mut Cx) {
        let m = original.as_ref().and_then(|d| self.pending.mounts.iter().find(|m| &m.destination == d)).cloned();
        let title = match &m {
            Some(m) => format!("Edit mount · {}", m.destination),
            None => "New mount".into(),
        };
        let kind_idx = match m.as_ref().map(|m| &m.source) {
            Some(MountSource::Git(_)) => 1,
            _ => 0,
        };
        let iso_idx = match m.as_ref().map(|m| m.isolation) {
            Some(Isolation::Worktree) => 1,
            Some(Isolation::Clone) => 2,
            _ => 0,
        };
        let f = self.ids.form;
        let mut fields = vec![
            FormField::radio("kind", RadioGroup::new(f.sub("kind"), "Source", &["Host directory", "Git repository"], kind_idx)),
            FormField::chooser(
                "host",
                f.sub("browse"),
                "Host directory",
                &m.as_ref().map(|m| match &m.source {
                    MountSource::Host(p) => w.tilde(p),
                    _ => String::new(),
                }).filter(|s| !s.is_empty()).unwrap_or("not chosen".into()),
                "Browse…",
            ),
            FormField::input(
                "git",
                TextInput::new(f.sub("git"), "Git URL").placeholder("github.com/org/repo").value(match m.as_ref().map(|m| &m.source) {
                    Some(MountSource::Git(u)) => u.as_str(),
                    _ => "",
                }),
            )
            .hidden(),
            FormField::input(
                "dest",
                TextInput::new(f.sub("dest"), "Destination").required(true).placeholder("/workspace/name").help("Absolute path inside the Construct").value(m.as_ref().map(|m| m.destination.as_str()).unwrap_or("")),
            ),
            FormField::check("ro", Checkbox::new(f.sub("ro"), "Mount read-only", m.as_ref().is_some_and(|m| m.readonly))),
            FormField::select("iso", Select::new(f.sub("iso"), "Isolation", &["Shared · edits land on the host", "Worktree · git worktree per instance", "Clone · full copy per instance"], iso_idx)),
        ];
        if self.scope == Scope::Global {
            let opts = self.scope_options();
            let refs: Vec<&str> = opts.iter().map(String::as_str).collect();
            let idx = match m.as_ref().map(|m| &m.scope) {
                Some(MountScope::Role(r)) => 1 + self.roles.iter().position(|x| x == r).unwrap_or(0),
                _ => 0,
            };
            fields.push(FormField::select("scope", Select::new(f.sub("scope"), "Scope", &refs, idx)));
        }
        let mut form = FormDialog::new(f, &title, fields).width(66).meta(if m.is_some() { "mount · edits" } else { "mount · new" });
        form.set_visible("git", kind_idx == 1);
        form.set_visible("host", kind_idx == 0);
        self.editing = Some(Editing::Mount { original });
        cx.open(Modal::Form(form), ModalTag::new("cfg.mount"));
    }

    fn open_env_form(&mut self, role: Option<RoleName>, original: Option<String>, w: &World, cx: &mut Cx) {
        let e = original.as_ref().and_then(|k| self.pending.env_of(role.as_deref()).iter().find(|e| &e.key == k)).cloned();
        let scope_word = role.as_deref().map(|r| format!("role {r}")).unwrap_or(self.scope.label().into());
        let title = match (&e, &role) {
            (Some(e), _) => format!("Edit {}", e.key),
            (None, None) => format!("New {} environment key", self.scope.label()),
            (None, Some(r)) => format!("New {r} environment key"),
        };
        let src_idx = match e.as_ref().map(|e| &e.value) {
            Some(EnvValue::OnePassword(_)) => 1,
            Some(EnvValue::HostEnv(_)) => 2,
            _ => 0,
        };
        let f = self.ids.form;
        self.op_ref = match e.as_ref().map(|e| &e.value) {
            Some(EnvValue::OnePassword(r)) => Some(r.clone()),
            _ => None,
        };
        let mut fields = vec![
            FormField::input("key", TextInput::new(f.sub("key"), "Key").required(true).placeholder("DATABASE_URL").validator(|v| env_key_error(v)).value(e.as_ref().map(|e| e.key.as_str()).unwrap_or(""))),
            FormField::radio("src", RadioGroup::new(f.sub("src"), "Value source", &["Plain text  (masked)", "1Password item / field  (recommended for secrets)", "Host environment variable"], src_idx)),
            FormField::input(
                "value",
                TextInput::new(f.sub("value"), &format!("Value for {}", e.as_ref().map(|e| e.key.as_str()).unwrap_or("the key")))
                    .masked()
                    .reveal_tail(4)
                    .placeholder("typed once · masked afterwards")
                    .value(match e.as_ref().map(|e| &e.value) {
                        Some(EnvValue::Plain(v)) => v.as_str(),
                        _ => "",
                    }),
            ),
            FormField::chooser("op", f.sub("op"), "1Password reference", &self.op_ref.as_ref().map(|r| r.display_path()).unwrap_or("not chosen".into()), "Choose…").hidden(),
            FormField::input(
                "host",
                TextInput::new(f.sub("host"), "Host variable").placeholder("GH_TOKEN").help("Forwarded from the host shell at launch").value(match e.as_ref().map(|e| &e.value) {
                    Some(EnvValue::HostEnv(h)) => h.as_str(),
                    _ => "",
                }),
            )
            .hidden(),
            FormField::note("scope", vec![(format!("Scope  {scope_word}"), Tone::Muted)]),
        ];
        if e.is_some() {
            // the key is the row identity; editing it re-creates the row
            if let Some(fld) = fields.iter_mut().find(|x| x.name == "key")
                && let FieldKindW::Input(i) = &mut fld.kind
            {
                i.help = "Renaming replaces the row".into();
            }
        }
        let mut form = FormDialog::new(f, &title, fields).width(70).meta("environment");
        form.set_visible("value", src_idx == 0);
        form.set_visible("op", src_idx == 1);
        form.set_visible("host", src_idx == 2);
        self.editing = Some(Editing::Env { role, original });
        let _ = w;
        cx.open(Modal::Form(form), ModalTag::new("cfg.env"));
    }

    fn open_auth_form(&mut self, role: Option<RoleName>, original: Option<Agent>, w: &World, cx: &mut Cx) {
        let e = original.and_then(|a| self.pending.auth_of(role.as_deref()).iter().find(|e| e.agent == a)).cloned();
        let agent = original.unwrap_or(Agent::ClaudeCode);
        let title = match (&e, original) {
            (Some(e), _) => format!("Edit auth · {}", e.agent.label()),
            (None, Some(a)) => format!("Auth override · {}", a.label()),
            (None, None) => "New auth override".into(),
        };
        let agents: Vec<&str> = Agent::ALL.iter().map(|a| a.label()).collect();
        let agent_idx = Agent::ALL.iter().position(|a| *a == agent).unwrap_or(0);
        let modes = agent.auth_modes();
        let mode_labels: Vec<&str> = modes.iter().map(|m| mode_label(*m)).collect();
        let mode_idx = e.as_ref().and_then(|e| modes.iter().position(|m| *m == e.mode)).unwrap_or(0);
        let src_idx = match e.as_ref().map(|e| &e.source) {
            Some(AuthSource::Account(_)) => 0,
            Some(AuthSource::Folder(_)) => 1,
            Some(AuthSource::OnePassword(_)) => 2,
            Some(AuthSource::Plain { .. }) => 3,
            _ => 4,
        };
        let f = self.ids.form;
        let accounts: Vec<String> = w.accounts.by_provider(agent.provider()).filter(|a| a.enabled).map(|a| format!("{}{}", a.display_name, if a.default_for_provider { " ★" } else { "" })).collect();
        let acc_refs: Vec<&str> = if accounts.is_empty() { vec!["no registered account · add one in Accounts (c)"] } else { accounts.iter().map(String::as_str).collect() };
        let acc_idx = match e.as_ref().map(|e| &e.source) {
            Some(AuthSource::Account(id)) => w.accounts.by_provider(agent.provider()).filter(|a| a.enabled).position(|a| &a.id == id).unwrap_or(0),
            _ => 0,
        };
        self.op_ref = match e.as_ref().map(|e| &e.source) {
            Some(AuthSource::OnePassword(r)) => Some(r.clone()),
            _ => None,
        };
        let fields = vec![
            FormField::select("agent", Select::new(f.sub("agent"), "Agent runtime", &agents, agent_idx).disabled(original.is_some())),
            FormField::note(
                "link",
                vec![(
                    format!("Provider {} · usage surface {}", agent.provider().label(), agent.usage_surface().surface_name()),
                    Tone::Muted,
                )],
            ),
            FormField::select("mode", Select::new(f.sub("mode"), "Mode", &mode_labels, mode_idx).help("sync mirrors the host profile · api key and oauth token take material from the source")),
            FormField::radio("src", RadioGroup::new(f.sub("src"), "Credential source", &["Registered account  (Accounts)", "Local agent folder", "1Password item / field", "Plain-text key  (masked)", "Host profile"], src_idx)),
            FormField::select("account", Select::new(f.sub("account"), "Account", &acc_refs, acc_idx).disabled(accounts.is_empty())),
            FormField::input(
                "folder",
                TextInput::new(f.sub("folder"), "Folder").placeholder("~/.claude").value(match e.as_ref().map(|e| &e.source) {
                    Some(AuthSource::Folder(p)) => p.as_str(),
                    _ => "",
                }),
            )
            .hidden(),
            FormField::chooser("browse", f.sub("browse"), "", "", "Browse…").hidden(),
            FormField::chooser("op", f.sub("op"), "1Password reference", &self.op_ref.as_ref().map(|r| r.display_path()).unwrap_or("not chosen".into()), "Choose…").hidden(),
            FormField::input("key", TextInput::new(f.sub("key"), "API key").masked().reveal_tail(4).placeholder("paste the key · shown masked").help("Only a fingerprint is stored")).hidden(),
            FormField::note("note", vec![]),
        ];
        let mut form = FormDialog::new(f, &title, fields).width(70).meta(role.as_deref().map(|r| format!("role {r}")).unwrap_or(self.scope.label().into()).as_str());
        Self::reveal_auth(&mut form);
        self.editing = Some(Editing::Auth { role, original });
        cx.open(Modal::Form(form), ModalTag::new("cfg.auth"));
    }

    fn reveal_auth(form: &mut FormDialog) {
        let src = form.choice("src");
        let mode_ignore = form.field("mode").map(|f| matches!(&f.kind, FieldKindW::Select(s) if s.value() == "ignore")).unwrap_or(false);
        form.set_visible("src", !mode_ignore);
        form.set_visible("account", !mode_ignore && src == 0);
        form.set_visible("folder", !mode_ignore && src == 1);
        form.set_visible("browse", !mode_ignore && src == 1);
        form.set_visible("op", !mode_ignore && src == 2);
        form.set_visible("key", !mode_ignore && src == 3);
        form.set_note(
            "note",
            if mode_ignore {
                vec![("ignore: the agent starts without credentials".into(), Tone::Muted)]
            } else if src == 4 {
                vec![("host profile: the host's own agent login is mirrored in".into(), Tone::Muted)]
            } else {
                vec![]
            },
        );
    }

    pub fn form_changed(&mut self, tag: &ModalTag, form: &mut FormDialog, _w: &World) {
        match tag.kind {
            "cfg.mount" => {
                let kind = form.choice("kind");
                form.set_visible("git", kind == 1);
                form.set_visible("host", kind == 0);
                form.error = None;
            }
            "cfg.env" => {
                let src = form.choice("src");
                form.set_visible("value", src == 0);
                form.set_visible("op", src == 1);
                form.set_visible("host", src == 2);
                form.error = None;
            }
            "cfg.auth" => {
                Self::reveal_auth(form);
                form.error = None;
            }
            _ => {}
        }
    }

    /// Handles results of the child modals this module opened. Returns
    /// `None` when the tag belongs to someone else.
    pub fn on_modal(&mut self, tag: &ModalTag, result: ModalResult, w: &mut World, cx: &mut Cx) -> Option<Outcome> {
        if !tag.kind.starts_with("cfg.") {
            return None;
        }
        let out = match (tag.kind, result) {
            ("cfg.mount", ModalResult::FormAction(name, _values)) => {
                if name == "choose:host" {
                    let b = FileBrowser::new(self.ids.form.sub("browser"), "Choose the host directory", &w.cwd, false, true, w);
                    cx.open(Modal::Browser(b), ModalTag::new("cfg.mount.browse"));
                }
                Outcome::Changed
            }
            ("cfg.mount.browse", ModalResult::Browser(BrowserResult::Chosen { path, .. })) => {
                let tilde = w.tilde(&path);
                let base = path.rsplit('/').next().unwrap_or("mount").to_owned();
                cx.with_form(move |f| {
                    f.set_chooser("host", &tilde, None);
                    if f.text("dest").trim().is_empty() {
                        f.set_text("dest", &format!("/workspace/{base}"));
                    }
                });
                Outcome::Changed
            }
            ("cfg.mount.browse", ModalResult::Browser(BrowserResult::GitUrl(url))) => {
                cx.with_form(move |f| {
                    if let Some(fld) = f.field_mut("kind")
                        && let FieldKindW::Radio(r) = &mut fld.kind
                    {
                        r.selected = 1;
                    }
                    f.set_visible("git", true);
                    f.set_visible("host", false);
                    f.set_text("git", &url);
                });
                Outcome::Changed
            }
            ("cfg.mount", ModalResult::Form(Some(values))) => self.save_mount(values, w, cx),
            ("cfg.env", ModalResult::FormAction(name, _)) => {
                if name == "choose:op" {
                    let flow = OpFlow::new(self.ids.form.sub("op"), &w.op, w.now_ms());
                    cx.open(Modal::Op(flow), ModalTag::new("cfg.env.opform"));
                }
                Outcome::Changed
            }
            ("cfg.env.opform", ModalResult::Op(Some(r))) => {
                let path = r.display_path();
                let masked = w.op.describe(&r).map(|d| d.masked).unwrap_or("••••••••".into());
                self.op_ref = Some(r);
                cx.with_form(move |f| f.set_chooser("op", &path, Some(&format!("value {masked} · resolved at launch"))));
                Outcome::Changed
            }
            ("cfg.env.op", ModalResult::Op(Some(r))) => {
                // direct re-pick from the row (`p`)
                if let Some(Editing::Env { role, original: Some(k) }) = self.editing.clone() {
                    if let Some(e) = self.pending.env_of_mut(role.as_deref()).iter_mut().find(|e| e.key == k) {
                        e.value = EnvValue::OnePassword(r.clone());
                        cx.status(format!("{k} now resolves from {}", r.display_path()));
                    }
                }
                self.editing = None;
                Outcome::Changed
            }
            ("cfg.env", ModalResult::Form(Some(values))) => self.save_env(values, w, cx),
            ("cfg.env.remove", ModalResult::Dialog { action: Some(1), .. }) => {
                let (role, key) = tag.key.split_once('\u{1}').map(|(a, b)| (if a.is_empty() { None } else { Some(a.to_owned()) }, b.to_owned())).unwrap_or((None, tag.key.clone()));
                self.pending.env_of_mut(role.as_deref()).retain(|e| e.key != key);
                cx.status(format!("Removed {key} · u restores until save"));
                Outcome::Changed
            }
            ("cfg.auth", ModalResult::FormAction(name, _)) => {
                match name.as_str() {
                    "choose:op" => {
                        let flow = OpFlow::new(self.ids.form.sub("op"), &w.op, w.now_ms());
                        cx.open(Modal::Op(flow), ModalTag::new("cfg.auth.op"));
                    }
                    "choose:browse" => {
                        let b = FileBrowser::new(self.ids.form.sub("browser"), "Choose the agent folder", &w.home, false, true, w);
                        cx.open(Modal::Browser(b), ModalTag::new("cfg.auth.browse"));
                    }
                    _ => {}
                }
                Outcome::Changed
            }
            ("cfg.auth.op", ModalResult::Op(Some(r))) => {
                let path = r.display_path();
                let masked = w.op.describe(&r).map(|d| d.masked).unwrap_or("••••••••".into());
                self.op_ref = Some(r);
                cx.with_form(move |f| f.set_chooser("op", &path, Some(&format!("value {masked} · resolved at launch"))));
                Outcome::Changed
            }
            ("cfg.auth.browse", ModalResult::Browser(BrowserResult::Chosen { path, .. })) => {
                let tilde = w.tilde(&path);
                cx.with_form(move |f| f.set_text("folder", &tilde));
                Outcome::Changed
            }
            ("cfg.auth", ModalResult::Form(Some(values))) => self.save_auth(values, w, cx),
            ("cfg.scope", ModalResult::Choice(Some(i))) => {
                let scope = if i == 0 { None } else { self.roles.get(i - 1).cloned() };
                let parts: Vec<&str> = tag.key.split('\u{1}').collect();
                match parts.as_slice() {
                    ["m", d] => {
                        if let Some(m) = self.pending.mounts.iter_mut().find(|m| &m.destination == d) {
                            m.scope = scope.clone().map(MountScope::Role).unwrap_or(MountScope::Global);
                            cx.status(format!("{d} · scope {}", m.scope.label()));
                        }
                    }
                    ["e", r, k] => {
                        let from = if r.is_empty() { None } else { Some((*r).to_owned()) };
                        if from != scope {
                            let list = self.pending.env_of_mut(from.as_deref());
                            if let Some(i) = list.iter().position(|e| &e.key == k) {
                                let e = list.remove(i);
                                let target = self.pending.env_of_mut(scope.as_deref());
                                target.retain(|x| x.key != e.key);
                                target.push(e);
                                cx.status(format!("{k} moved to {}", scope.as_deref().map(|r| format!("role {r}")).unwrap_or(self.scope.label().into())));
                            }
                        }
                    }
                    _ => {}
                }
                Outcome::Changed
            }
            (_, ModalResult::Form(None)) => {
                self.editing = None;
                cx.status("Cancelled · nothing changed");
                Outcome::Changed
            }
            _ => Outcome::Changed,
        };
        Some(out)
    }

    fn save_mount(&mut self, values: FormValues, w: &World, cx: &mut Cx) -> Outcome {
        let dest = text(&values, "dest").trim().to_owned();
        if !dest.starts_with('/') {
            cx.with_form(|f| f.error = Some("Destination must be an absolute path inside the Construct".into()));
            return Outcome::Changed;
        }
        let kind = choice(&values, "kind");
        let source = if kind == 1 {
            let url = text(&values, "git").trim().trim_start_matches("https://").to_owned();
            if url.is_empty() || !url.contains('/') {
                cx.with_form(|f| f.error = Some("Git URL is required (host/org/repo)".into()));
                return Outcome::Changed;
            }
            MountSource::Git(url)
        } else {
            let host = chooser_value(&values, "host");
            if host.is_empty() || host == "not chosen" {
                cx.with_form(|f| f.error = Some("Choose the host directory first".into()));
                return Outcome::Changed;
            }
            MountSource::Host(crate::sim::world::expand(&w.home, &host))
        };
        let original = match &self.editing {
            Some(Editing::Mount { original }) => original.clone(),
            _ => None,
        };
        if self.pending.mounts.iter().any(|m| m.destination == dest && Some(&m.destination) != original.as_ref()) {
            cx.with_form(move |f| f.error = Some(format!("{dest} is already a mount destination")));
            return Outcome::Changed;
        }
        let iso = match choice(&values, "iso") {
            1 => Isolation::Worktree,
            2 => Isolation::Clone,
            _ => Isolation::Shared,
        };
        let scope = if self.scope == Scope::Global {
            let i = choice(&values, "scope");
            if i == 0 { MountScope::Global } else { self.roles.get(i - 1).cloned().map(MountScope::Role).unwrap_or(MountScope::Global) }
        } else {
            MountScope::Workspace
        };
        let mut m = match original.as_ref().and_then(|d| self.pending.mounts.iter().find(|m| &m.destination == d)).cloned() {
            Some(m) => m,
            None => Mount::host("", &dest),
        };
        let is_git = matches!(source, MountSource::Git(_));
        m.source = source;
        m.destination = dest.clone();
        m.readonly = checked(&values, "ro");
        m.isolation = iso;
        m.kind = if is_git || m.kind == MountKind::Repository { MountKind::Repository } else { MountKind::Directory };
        m.scope = scope;
        m.drift = None;
        match original {
            Some(d) => {
                if let Some(slot) = self.pending.mounts.iter_mut().find(|x| x.destination == d) {
                    *slot = m;
                }
                cx.status(format!("Mount {dest} updated · save to apply"));
            }
            None => {
                self.pending.mounts.push(m);
                cx.status(format!("Mount {dest} added · save to apply"));
            }
        }
        self.editing = None;
        let st = &mut self.mounts;
        st.cursor = self.pending.mounts.iter().position(|m| m.destination == dest).unwrap_or(0);
        cx.close();
        Outcome::Changed
    }

    fn save_env(&mut self, values: FormValues, _w: &World, cx: &mut Cx) -> Outcome {
        let key = text(&values, "key").trim().to_owned();
        if let Some(e) = env_key_error(&key) {
            cx.with_form(move |f| f.error = Some(e));
            return Outcome::Changed;
        }
        let value = match choice(&values, "src") {
            0 => {
                let v = text(&values, "value");
                if v.is_empty() {
                    cx.with_form(|f| f.error = Some("Value cannot be empty".into()));
                    return Outcome::Changed;
                }
                EnvValue::Plain(v)
            }
            1 => match self.op_ref.clone() {
                Some(r) => EnvValue::OnePassword(r),
                None => {
                    cx.with_form(|f| f.error = Some("Choose a 1Password item and field first".into()));
                    return Outcome::Changed;
                }
            },
            _ => {
                let h = text(&values, "host").trim().to_owned();
                if let Some(e) = env_key_error(&h) {
                    cx.with_form(move |f| f.error = Some(format!("Host variable: {e}")));
                    return Outcome::Changed;
                }
                EnvValue::HostEnv(h)
            }
        };
        let Some(Editing::Env { role, original }) = self.editing.clone() else { return Outcome::Changed };
        let list = self.pending.env_of_mut(role.as_deref());
        if let Some(o) = &original {
            list.retain(|e| &e.key != o);
        }
        if list.iter().any(|e| e.key == key) {
            cx.with_form(move |f| f.error = Some(format!("{key} already exists in this scope")));
            return Outcome::Changed;
        }
        list.push(EnvVar { key: key.clone(), value });
        self.unmasked.remove(&(role.clone(), key.clone()));
        self.editing = None;
        self.op_ref = None;
        cx.status(format!("{key} · {} · save to apply", if original.is_some() { "updated" } else { "added" }));
        cx.close();
        Outcome::Changed
    }

    fn save_auth(&mut self, values: FormValues, w: &World, cx: &mut Cx) -> Outcome {
        let agent = Agent::ALL[choice(&values, "agent").min(Agent::ALL.len() - 1)];
        let modes = agent.auth_modes();
        let mode = modes[choice(&values, "mode").min(modes.len() - 1)];
        let source = if mode == AuthMode::Ignore {
            AuthSource::None
        } else {
            match choice(&values, "src") {
                0 => {
                    let accounts: Vec<String> = w.accounts.by_provider(agent.provider()).filter(|a| a.enabled).map(|a| a.id.clone()).collect();
                    match accounts.get(choice(&values, "account")) {
                        Some(id) => AuthSource::Account(id.clone()),
                        None => {
                            cx.with_form(|f| f.error = Some("No registered account for this agent · add one in Accounts (c) or pick another source".into()));
                            return Outcome::Changed;
                        }
                    }
                }
                1 => {
                    let p = text(&values, "folder").trim().to_owned();
                    if p.is_empty() {
                        cx.with_form(|f| f.error = Some("Choose or type the agent folder".into()));
                        return Outcome::Changed;
                    }
                    AuthSource::Folder(p)
                }
                2 => match self.op_ref.clone() {
                    Some(r) => AuthSource::OnePassword(r),
                    None => {
                        cx.with_form(|f| f.error = Some("Choose a 1Password item and field first".into()));
                        return Outcome::Changed;
                    }
                },
                3 => {
                    let k = text(&values, "key");
                    if k.trim().is_empty() {
                        cx.with_form(|f| f.error = Some("API key required".into()));
                        return Outcome::Changed;
                    }
                    AuthSource::Plain {
                        fingerprint: crate::domain::account::fingerprint(&k),
                    }
                }
                _ => AuthSource::HostProfile,
            }
        };
        let Some(Editing::Auth { role, original }) = self.editing.clone() else { return Outcome::Changed };
        let list = self.pending.auth_of_mut(role.as_deref());
        if original.is_none() && list.iter().any(|e| e.agent == agent) {
            cx.with_form(move |f| f.error = Some(format!("{} already has an override in this scope · edit that row", agent.label())));
            return Outcome::Changed;
        }
        list.retain(|e| e.agent != agent);
        list.push(AuthEntry { agent, mode, source });
        self.editing = None;
        self.op_ref = None;
        cx.status(format!("{} · {} · save to apply", agent.label(), mode_label(mode)));
        cx.close();
        Outcome::Changed
    }
}

// ------------------------------------------------------------------ util

fn diff_len<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    a.iter().filter(|x| !b.contains(x)).count() + b.iter().filter(|x| !a.contains(x)).count()
}

fn counts<T: PartialEq>(pending: &[T], original: &[T], key: impl Fn(&T) -> String) -> (usize, usize, usize) {
    let mut added = 0;
    let mut modified = 0;
    for p in pending {
        match original.iter().find(|o| key(o) == key(p)) {
            None => added += 1,
            Some(o) if o != p => modified += 1,
            _ => {}
        }
    }
    let removed = original.iter().filter(|o| !pending.iter().any(|p| key(p) == key(o))).count();
    (added, modified, removed)
}

fn column_widths(tab: Tab, scope: Scope, avail: u16) -> Vec<u16> {
    match (tab, scope) {
        (Tab::Mounts, Scope::Workspace) => {
            // Destination, Mode, Isolation, Kind, Source
            let fixed = 4 + 9 + 6;
            if avail >= 90 {
                let rest = avail.saturating_sub(fixed + 8);
                let dest = (rest * 45 / 100).max(20);
                vec![dest, 4, 9, 6, rest.saturating_sub(dest)]
            } else if avail >= 60 {
                vec![avail.saturating_sub(fixed + 6), 4, 9, 6, 0]
            } else {
                vec![avail.saturating_sub(4 + 9 + 4), 4, 9, 0, 0]
            }
        }
        (Tab::Mounts, Scope::Global) => {
            let fixed = 14 + 4 + 9 + 6;
            if avail >= 100 {
                let rest = avail.saturating_sub(fixed + 10);
                let dest = (rest * 45 / 100).max(20);
                vec![dest, 14, 4, 9, 6, rest.saturating_sub(dest)]
            } else if avail >= 70 {
                vec![avail.saturating_sub(fixed + 8), 14, 4, 9, 6, 0]
            } else {
                vec![avail.saturating_sub(14 + 4 + 9 + 6), 14, 4, 9, 0, 0]
            }
        }
        (Tab::Environments, Scope::Workspace) => {
            let key = 18.min(avail / 3);
            let value = if avail >= 90 { 24 } else { 18 };
            vec![key, value, avail.saturating_sub(key + value + 4)]
        }
        (Tab::Environments, Scope::Global) => {
            let key = 18.min(avail / 3);
            let value = if avail >= 100 { 22 } else { 16 };
            vec![key, 14, value, avail.saturating_sub(key + 14 + value + 6)]
        }
        (Tab::Auth, Scope::Workspace) => {
            let (a, m, s) = (13, 11, 10);
            vec![a, m, s, avail.saturating_sub(a + m + s + 6)]
        }
        (Tab::Auth, Scope::Global) => {
            let (a, sc, m, s) = (13, 14, 11, 10);
            vec![a, sc, m, s, avail.saturating_sub(a + sc + m + s + 8)]
        }
    }
}

fn text(values: &FormValues, name: &str) -> String {
    values
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| match v {
            FieldValue::Text(s) => s.clone(),
            _ => String::new(),
        })
        .unwrap_or_default()
}

fn chooser_value(values: &FormValues, name: &str) -> String {
    values
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| match v {
            FieldValue::Text(s) => s.clone(),
            _ => String::new(),
        })
        .unwrap_or_default()
}

fn choice(values: &FormValues, name: &str) -> usize {
    values
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| match v {
            FieldValue::Choice(i) => *i,
            _ => 0,
        })
        .unwrap_or(0)
}

fn checked(values: &FormValues, name: &str) -> bool {
    values
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| matches!(v, FieldValue::Bool(true)))
        .unwrap_or(false)
}

pub fn mode_label(m: AuthMode) -> &'static str {
    match m {
        AuthMode::Sync => "sync",
        AuthMode::ApiKey => "api key",
        AuthMode::OAuthToken => "oauth token",
        AuthMode::Ignore => "ignore",
    }
}

pub fn source_kind(s: &AuthSource) -> &'static str {
    match s {
        AuthSource::HostProfile => "host profile",
        AuthSource::Folder(_) => "folder",
        AuthSource::Account(_) => "account",
        AuthSource::OnePassword(_) => "1Password",
        AuthSource::Plain { .. } => "plain",
        AuthSource::None => "",
    }
}

pub fn source_detail(s: &AuthSource, w: &World) -> String {
    match s {
        AuthSource::HostProfile => "host login mirrored".into(),
        AuthSource::Folder(p) => w.tilde(p),
        AuthSource::Account(id) => match w.accounts.get(id) {
            Some(a) => {
                let mut t = a.title();
                if a.default_for_provider {
                    t.push_str(" ★");
                }
                if let Some(e) = &a.endpoint {
                    t.push_str(&format!(" · endpoint {}", e.host));
                }
                t
            }
            None => format!("account {id} · not in registry"),
        },
        AuthSource::OnePassword(r) => r.display_path(),
        AuthSource::Plain { fingerprint } => format!("fingerprint {fingerprint}"),
        AuthSource::None => String::new(),
    }
}

/// What an agent uses when nothing overrides it: the provider default in
/// the registry, else the discovered host login, else nothing.
pub fn builtin_default(agent: Agent, w: &World) -> String {
    let p = agent.provider();
    if let Some(a) = w.accounts.default_for(p) {
        return format!("default ★ {}", a.title());
    }
    if let Some(a) = w.accounts.discovered_current(p) {
        return format!("discovered · {}", match &a.source {
            CredentialSource::LocalFolder { path, .. } => w.tilde(path),
            s => s.safe_detail(),
        });
    }
    "no credentials found".into()
}

fn mask_len(n: usize) -> String {
    "*".repeat(n)
}
