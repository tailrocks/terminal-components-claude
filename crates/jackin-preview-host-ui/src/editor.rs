//! Workspace editor state, public control ids, and EditorScreen component.

use core::{fmt, mem};
use std::collections::BTreeMap;

use ratatui::layout::Rect;
use termrock::author::{FgStep, Modifier, Part, Role, StylePatch, Ui};
use termrock::controls::{Button, Checkbox, Panel, PanelKind};
use termrock::fields::{Field, TextInput, TextInputState};
use termrock::layout::Track;
use termrock::navigation::{List, ListState, Tabs, TabsState};
use termrock::overlays::{Select, SelectState};
use termrock::{Hint, HintKey, HintLayer, Id, ItemKey, Span, Variant, truncate, width};

use jackin_preview_domain::account::{AccountId, AccountOrigin, AccountRegistry};
use jackin_preview_domain::agent::Provider;
use jackin_preview_domain::workspace::{
    AccountPolicy, AllowedRoles, DirtyExitPolicy, Effective, EffectiveAccount, EnvValue, EnvVar,
    Mount, MountSource, RoleName, RolePolicy, Workspace, env_key_error, mask, usability_of,
};
use jackin_preview_sim::world::World;

/// Editor root.
pub const ROOT: Id = Id::root("jackin.editor");
/// Editor form root retained as a stable namespace for nested controls.
pub const FORM: Id = ROOT.sub("form");
/// Legacy editor configuration root retained for save-form compatibility.
pub const CFG: Id = Id::root("editor.cfg");
/// Legacy editor configuration form root retained for save-form compatibility.
pub const CFG_FORM: Id = CFG.sub("form");
/// Save action.
pub const SAVE: Id = CFG_FORM.sub("save");
/// Editor tabs.
pub const TABS: Id = ROOT.sub("tabs");
/// General tab control id.
pub const TAB_GENERAL: Id = TABS.sub("general");
/// Mounts tab control id.
pub const TAB_MOUNTS: Id = TABS.sub("mounts");
/// Roles tab control id.
pub const TAB_ROLES: Id = TABS.sub("roles");
/// Environments tab control id.
pub const TAB_ENVIRONMENTS: Id = TABS.sub("environments");
/// Accounts tab control id.
pub const TAB_ACCOUNTS: Id = TABS.sub("accounts");
/// Keep-awake checkbox on the General tab.
pub const KEEP_AWAKE: Id = FORM.sub("keep-awake");
/// Git-pull checkbox on the General tab.
pub const GIT_PULL: Id = FORM.sub("git-pull");
/// New environment-variable key input.
pub const ENV_KEY: Id = FORM.sub("env-key");
/// New environment-variable source selector.
pub const ENV_SOURCE: Id = FORM.sub("env-source");
/// New environment-variable value input.
pub const ENV_VALUE: Id = FORM.sub("env-value");
/// Workspace name field on the General tab.
pub const NAME: Id = FORM.sub("name");
/// Mounts body list.
pub const MOUNTS_LIST: Id = FORM.sub("mounts-list");
/// Roles body list.
pub const ROLES_LIST: Id = FORM.sub("roles-list");
/// Environments body list.
pub const ENV_LIST: Id = FORM.sub("env-list");
/// Accounts body list.
pub const ACCOUNTS_LIST: Id = FORM.sub("accounts-list");
/// Save-preview dialog Cancel action (holds initial dialog focus).
pub const PREVIEW_CANCEL: Id = CFG_FORM.sub("cancel");
/// Dirty-exit dialog root.
pub const EXIT: Id = ROOT.sub("exit");
/// Dirty-exit dialog Cancel action (holds initial dialog focus).
pub const EXIT_CANCEL: Id = EXIT.sub("cancel");
/// Dirty-exit dialog Discard action.
pub const EXIT_DISCARD: Id = EXIT.sub("discard");
/// Dirty-exit dialog Save action.
pub const EXIT_SAVE: Id = EXIT.sub("save");

/// Editor tab projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// General workspace properties.
    #[default]
    General,
    /// Mounts.
    Mounts,
    /// Roles.
    Roles,
    /// Environment variables.
    Environments,
    /// Account policy.
    Accounts,
}

impl Tab {
    /// Canonical editor tab order. Keep this in one place so keyboard and
    /// pointer navigation cannot drift apart.
    pub const ALL: [Self; 5] = [
        Self::General,
        Self::Mounts,
        Self::Roles,
        Self::Environments,
        Self::Accounts,
    ];

    /// Convert a one-based public tab alias to a tab.
    pub const fn from_alias(index: u8) -> Option<Self> {
        match index {
            1 => Some(Self::General),
            2 => Some(Self::Mounts),
            3 => Some(Self::Roles),
            4 => Some(Self::Environments),
            5 => Some(Self::Accounts),
            _ => None,
        }
    }

    /// Return the one-based public tab alias.
    pub const fn alias(self) -> u8 {
        match self {
            Self::General => 1,
            Self::Mounts => 2,
            Self::Roles => 3,
            Self::Environments => 4,
            Self::Accounts => 5,
        }
    }

    /// Return the next tab, wrapping at the end of the strip.
    pub const fn next(self) -> Self {
        match self {
            Self::General => Self::Mounts,
            Self::Mounts => Self::Roles,
            Self::Roles => Self::Environments,
            Self::Environments => Self::Accounts,
            Self::Accounts => Self::General,
        }
    }

    /// Return the previous tab, wrapping at the beginning of the strip.
    pub const fn previous(self) -> Self {
        match self {
            Self::General => Self::Accounts,
            Self::Mounts => Self::General,
            Self::Roles => Self::Mounts,
            Self::Environments => Self::Roles,
            Self::Accounts => Self::Environments,
        }
    }

    /// Stable control id for this tab.
    pub const fn id(self) -> Id {
        match self {
            Self::General => TAB_GENERAL,
            Self::Mounts => TAB_MOUNTS,
            Self::Roles => TAB_ROLES,
            Self::Environments => TAB_ENVIRONMENTS,
            Self::Accounts => TAB_ACCOUNTS,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct SaveReview {
    original: Option<Box<Workspace>>,
    pending: PendingWorkspace,
}

/// Whether keyboard focus sits on the tab strip or inside the tab body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorFocus {
    /// The tab strip holds focus (digits jump, Enter moves into the body).
    #[default]
    Tabs,
    /// A body control holds focus (Esc refocuses the tab strip).
    Body,
}

/// Durable editor state.
#[derive(PartialEq, Eq, Default)]
pub struct EditorState {
    /// Active editor tab.
    pub tab: Tab,
    /// Tab-strip versus body focus.
    pub focus: EditorFocus,
    /// Whether the draft has unsaved changes.
    pub dirty: bool,
    /// Whether the read-only preview is open.
    pub preview_open: bool,
    /// Whether the dirty-exit dialog is open.
    pub exit_open: bool,
    /// Whether the environment-variable form is open.
    pub env_form_open: bool,
    /// Draft environment-variable key.
    pub env_key: String,
    /// Draft environment-variable value; never rendered unmasked.
    pub env_value: String,
    /// Runtime state for the key input.
    pub env_key_input: termrock::TextInputState,
    /// Sensitive runtime state for the value input.
    pub env_value_input: termrock::TextInputState,
    /// Mutable workspace draft projected by the editor controls.
    pub pending: PendingWorkspace,
    original: Option<Box<Workspace>>,
    reviewed: Option<SaveReview>,
    saving: Option<jackin_preview_domain::workspace_save::SaveTicket>,
}

impl Clone for EditorState {
    fn clone(&self) -> Self {
        Self {
            tab: self.tab,
            focus: self.focus,
            dirty: self.dirty,
            preview_open: self.preview_open,
            exit_open: self.exit_open,
            env_form_open: self.env_form_open,
            env_key: self.env_key.clone(),
            // A cloned editor is a safe snapshot, not a continuation that
            // copies an in-flight environment secret.
            env_value: String::new(),
            env_key_input: self.env_key_input.clone(),
            env_value_input: termrock::TextInputState::sensitive(),
            pending: self.pending.clone(),
            original: self.original.clone(),
            reviewed: self.reviewed.clone(),
            saving: self.saving,
        }
    }
}

impl fmt::Debug for EditorState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorState")
            .field("tab", &self.tab)
            .field("focus", &self.focus)
            .field("dirty", &self.dirty)
            .field("preview_open", &self.preview_open)
            .field("exit_open", &self.exit_open)
            .field("env_form_open", &self.env_form_open)
            .field("env_key", &self.env_key)
            .field("env_value", &"[redacted]")
            .field("env_key_input", &self.env_key_input)
            .field("env_value_input", &self.env_value_input)
            .field("pending", &self.pending)
            .finish()
    }
}

impl Drop for EditorState {
    fn drop(&mut self) {
        self.env_value_input.zeroize();
        wipe_string(&mut self.env_value);
    }
}

impl EditorState {
    /// Select a tab by the legacy transition index used by the preview shell.
    pub const fn select_index(&mut self, index: u8) {
        self.tab = match (self.tab, index) {
            (Tab::Accounts, 1) => Tab::General,
            (_, 1) => Tab::Mounts,
            (_, 2) => Tab::Roles,
            (_, 3) => Tab::Environments,
            (_, 4 | 5) => Tab::Accounts,
            _ => Tab::General,
        };
    }

    /// Select a tab by its one-based public alias (`1 = General`, …).
    pub const fn select_alias(&mut self, index: u8) {
        if let Some(tab) = Tab::from_alias(index) {
            self.tab = tab;
        }
    }

    /// Advance to the next tab, wrapping at the strip boundary.
    pub const fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }

    /// Move to the previous tab, wrapping at the strip boundary.
    pub const fn previous_tab(&mut self) {
        self.tab = self.tab.previous();
    }

    /// Start a fresh editor draft from a persisted workspace.
    pub fn load_workspace(&mut self, workspace: &Workspace) {
        self.tab = Tab::General;
        self.focus = EditorFocus::Tabs;
        self.dirty = false;
        self.preview_open = false;
        self.exit_open = false;
        self.pending = PendingWorkspace::from_workspace(workspace);
        self.original = Some(Box::new(workspace.clone()));
        self.reviewed = None;
        self.saving = None;
        self.clear_env_form();
    }

    /// Mark the current draft as changed and close any stale preview.
    pub fn mark_dirty(&mut self) {
        self.reviewed = None;
        self.dirty = true;
        self.preview_open = false;
    }

    /// Return the count of unsaved changes.
    pub const fn change_count(&self) -> usize {
        if self.dirty { 1 } else { 0 }
    }

    /// Mark a successful save and close the preview.
    pub fn mark_saved(&mut self) {
        self.reviewed = None;
        self.dirty = false;
        self.preview_open = false;
    }

    /// Open the save preview only when there are pending changes.
    pub fn open_preview(&mut self) -> bool {
        if self.dirty && self.saving.is_none() {
            self.reviewed = Some(SaveReview {
                original: self.original.clone(),
                pending: self.pending.clone(),
            });
            self.preview_open = true;
        }
        self.preview_open
    }

    /// Close a save preview without discarding the draft.
    pub fn close_preview(&mut self) {
        self.preview_open = false;
        self.reviewed = None;
    }

    /// Open the dirty-exit dialog only when there are pending changes.
    pub fn open_exit(&mut self) -> bool {
        if self.dirty && self.saving.is_none() {
            self.exit_open = true;
        }
        self.exit_open
    }

    /// Close the dirty-exit dialog without discarding the draft.
    pub fn close_exit(&mut self) {
        self.exit_open = false;
    }

    /// Move keyboard focus to the tab strip, staying on the current tab.
    pub fn focus_tabs(&mut self) {
        self.focus = EditorFocus::Tabs;
    }

    /// Move keyboard focus into the current tab body.
    pub fn focus_body(&mut self) {
        self.focus = EditorFocus::Body;
    }

    /// Bind a new configuration explicitly, clearing any previous editor ticket.
    pub fn load_new(&mut self, pending: PendingWorkspace) {
        self.tab = Tab::General;
        self.focus = EditorFocus::Tabs;
        self.pending = pending;
        self.original = None;
        self.reviewed = None;
        self.saving = None;
        self.dirty = true;
        self.preview_open = false;
        self.exit_open = false;
        self.clear_env_form();
    }

    /// Whether this draft creates a configuration rather than replacing a loaded one.
    pub const fn is_create(&self) -> bool {
        self.original.is_none()
    }

    /// Original mount by destination for change comparison.
    pub fn original_mount(&self, dest: &str) -> Option<&Mount> {
        self.original
            .as_ref()
            .and_then(|o| o.mounts.iter().find(|m| m.destination == dest))
    }

    /// Whether this editor owns an admitted asynchronous save.
    pub const fn is_saving(&self) -> bool {
        self.saving.is_some()
    }

    /// Stable loaded workspace, independent of list order.
    pub fn workspace_id(&self) -> Option<jackin_preview_domain::workspace::WorkspaceId> {
        self.original.as_ref().map(|workspace| workspace.id)
    }

    /// Admit exactly the immutable preview; changed values require a fresh review.
    pub fn begin_save(
        &mut self,
        world: &mut jackin_preview_sim::world::World,
    ) -> Result<
        jackin_preview_domain::workspace_save::SaveTicket,
        jackin_preview_domain::workspace_save::SaveError,
    > {
        use jackin_preview_domain::workspace_save::SaveError;
        if self.saving.is_some() {
            return Err(SaveError::Busy);
        }
        if !self.preview_open {
            return Err(SaveError::NoReview);
        }
        let review = self.reviewed.take().ok_or(SaveError::NoReview)?;
        self.preview_open = false;
        if review.pending != self.pending || review.original != self.original {
            return Err(SaveError::ChangedReview);
        }
        let mut proposed = review
            .original
            .as_deref()
            .cloned()
            .unwrap_or_else(|| review.pending.clone().into_workspace(0));
        review.pending.apply_to(&mut proposed);
        let ticket = world.begin_editor_write(review.original.as_deref(), proposed)?;
        self.saving = Some(ticket);
        Ok(ticket)
    }

    /// Settle only this editor's ticket. True means its unchanged draft is now clean.
    pub fn settle_save(
        &mut self,
        result: &jackin_preview_domain::workspace_save::SaveResult,
    ) -> bool {
        use jackin_preview_domain::workspace_save::SaveResult;
        let Some(ticket) = result.ticket() else {
            return false;
        };
        if self.saving != Some(ticket) {
            return false;
        }
        self.saving = None;
        self.reviewed = None;
        self.preview_open = false;
        match result {
            SaveResult::Saved { workspace, .. } => {
                let unchanged = self.pending == PendingWorkspace::from_workspace(workspace);
                self.original = Some(workspace.clone());
                self.dirty = !unchanged;
                unchanged
            }
            SaveResult::Failed(_) | SaveResult::Stale(_) => {
                self.dirty = true;
                false
            }
            SaveResult::Ignored => false,
        }
    }

    /// Open a fresh environment-variable form, dropping any previous input.
    pub fn open_env_form(&mut self) {
        self.clear_env_form();
        self.env_form_open = true;
        self.env_value_input = termrock::TextInputState::sensitive();
    }

    /// Cancel and clear all transient environment-variable input.
    pub fn clear_env_form(&mut self) {
        self.env_form_open = false;
        self.env_key.clear();
        self.env_key_input = termrock::TextInputState::default();
        self.env_value_input.zeroize();
        self.env_value_input = termrock::TextInputState::default();
        wipe_string(&mut self.env_value);
    }

    /// Drop the transient value while retaining the rest of the form.
    pub fn discard_env_value(&mut self) {
        self.env_value_input.cancel();
        self.env_value_input.zeroize();
        wipe_string(&mut self.env_value);
    }

    /// Move the committed transient value into its durable draft owner.
    pub fn take_env_value(&mut self) -> String {
        self.env_value_input.zeroize();
        self.env_value_input = termrock::TextInputState::sensitive();
        mem::take(&mut self.env_value)
    }
}

fn wipe_string(value: &mut String) {
    let mut secret = termrock::Secret::new(mem::take(value));
    secret.zeroize();
}

/// Workspace draft owned by the editor route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingWorkspace {
    /// Proposed display name.
    pub name: String,
    /// Proposed working directory.
    pub workdir: String,
    /// Mount rows.
    pub mounts: Vec<Mount>,
    /// Role allow-list and preferred role.
    pub roles: RolePolicy,
    /// Workspace environment draft. Persisted only after the editor save job succeeds.
    pub env: Vec<EnvVar>,
    /// Role-scoped environment drafts keyed by role name.
    pub role_env: BTreeMap<RoleName, Vec<EnvVar>>,
    /// Account activation policy.
    pub accounts: AccountPolicy,
    /// Keep the workspace alive after its last session exits.
    pub keep_awake: bool,
    /// Pull the configured repository before starting a session.
    pub git_pull: bool,
    /// Policy for leaving with unsaved workspace changes.
    pub dirty_policy: DirtyExitPolicy,
}

impl Default for PendingWorkspace {
    fn default() -> Self {
        Self {
            name: "payments-platform".into(),
            workdir: "/workspace/payments-platform".into(),
            dirty_policy: DirtyExitPolicy::Ask,
            mounts: vec![
                Mount::host("/workspace/payments-platform", "~/src/payments-platform"),
                Mount::host("/workspace/libs", "~/src/shared-libs"),
            ],
            roles: RolePolicy::default(),
            env: vec![],
            role_env: BTreeMap::new(),
            accounts: AccountPolicy::default(),
            keep_awake: true,
            git_pull: true,
        }
    }
}

impl PendingWorkspace {
    /// Copy a persisted workspace into an editable draft.
    pub fn from_workspace(workspace: &Workspace) -> Self {
        Self {
            name: workspace.name.clone(),
            workdir: workspace.workdir.clone(),
            mounts: workspace.mounts.clone(),
            roles: workspace.roles.clone(),
            env: workspace.env.clone(),
            role_env: workspace.role_env.clone(),
            accounts: workspace.accounts.clone(),
            keep_awake: workspace.keep_awake,
            git_pull: workspace.git_pull,
            dirty_policy: workspace.dirty_policy,
        }
    }

    /// Apply this draft to a persisted workspace without changing its id.
    pub fn apply_to(&self, workspace: &mut Workspace) {
        workspace.name = self.name.clone();
        workspace.workdir = self.workdir.clone();
        workspace.mounts = self.mounts.clone();
        workspace.roles = self.roles.clone();
        workspace.env = self.env.clone();
        workspace.role_env = self.role_env.clone();
        workspace.accounts = self.accounts.clone();
        workspace.keep_awake = self.keep_awake;
        workspace.git_pull = self.git_pull;
        workspace.dirty_policy = self.dirty_policy;
    }

    /// Consume this draft into a persisted workspace with `id`.
    pub fn into_workspace(self, id: u32) -> Workspace {
        let mut workspace = Workspace::new(id, &self.name, &self.workdir);
        workspace.mounts = self.mounts;
        workspace.roles = self.roles;
        workspace.env = self.env;
        workspace.role_env = self.role_env;
        workspace.accounts = self.accounts;
        workspace.keep_awake = self.keep_awake;
        workspace.git_pull = self.git_pull;
        workspace.dirty_policy = self.dirty_policy;
        workspace
    }

    /// Effective accounts projected with current registry metadata.
    pub fn effective_accounts(&self, registry: &AccountRegistry) -> Vec<EffectiveAccount> {
        let mut workspace = Workspace::new(0, &self.name, &self.workdir);
        workspace.mounts = self.mounts.clone();
        workspace.roles = self.roles.clone();
        workspace.env = self.env.clone();
        workspace.role_env = self.role_env.clone();
        workspace.accounts = self.accounts.clone();
        workspace.effective_accounts(registry)
    }

    /// Set a proposed account as enabled in this workspace.
    pub fn enable_account(&mut self, id: impl Into<AccountId>) {
        let id = id.into();
        self.accounts.disabled_defaults.remove(&id);
        self.accounts.enabled.insert(id);
    }

    /// Disable an account in this workspace, if it is known to the registry.
    pub fn disable_account(
        &mut self,
        id: impl Into<AccountId>,
        registry: &AccountRegistry,
    ) -> Result<(), String> {
        let id = id.into();
        let Some(account) = registry.get(&id) else {
            return Err(format!("account {id} is not configured"));
        };
        self.accounts.enabled.remove(&id);
        if account.default_for_provider {
            self.accounts.disabled_defaults.insert(id.clone());
        }
        self.accounts
            .preferred
            .retain(|_, preferred| preferred != &id);
        self.accounts
            .role_preferred
            .retain(|_, preferred| preferred != &id);
        Ok(())
    }

    /// Toggle an account's workspace activation and return its new state.
    pub fn toggle_account(
        &mut self,
        id: impl Into<AccountId>,
        registry: &AccountRegistry,
    ) -> Result<bool, String> {
        let id = id.into();
        let active = self.effective_accounts(registry).iter().any(|a| a.id == id);
        if active {
            self.disable_account(id, registry)?;
            Ok(false)
        } else {
            let Some(account) = registry.get(&id) else {
                return Err(format!("account {id} is not configured"));
            };
            self.accounts.disabled_defaults.remove(&id);
            if account.default_for_provider {
                self.accounts.enabled.remove(&id);
            } else {
                self.enable_account(id);
            }
            Ok(true)
        }
    }

    /// Set the preferred account for its provider.
    pub fn prefer_account(
        &mut self,
        id: impl Into<AccountId>,
        registry: &AccountRegistry,
    ) -> Result<(), String> {
        let id = id.into();
        let Some(account) = registry.get(&id) else {
            return Err(format!("account {id} is not configured"));
        };
        if !self.effective_accounts(registry).iter().any(|a| a.id == id) {
            return Err(format!("account {id} is not active in this workspace"));
        }
        self.accounts.preferred.insert(account.provider, id);
        Ok(())
    }

    /// Set a role-specific preferred account for its provider.
    pub fn prefer_role_account(
        &mut self,
        role: impl Into<RoleName>,
        id: impl Into<AccountId>,
        registry: &AccountRegistry,
    ) -> Result<(), String> {
        let role = role.into();
        let id = id.into();
        let Some(account) = registry.get(&id) else {
            return Err(format!("account {id} is not configured"));
        };
        if !self.effective_accounts(registry).iter().any(|a| a.id == id) {
            return Err(format!("account {id} is not active in this workspace"));
        }
        self.accounts
            .role_preferred
            .insert((role, account.provider), id);
        Ok(())
    }

    /// Add a workspace-scoped plain environment variable after validation.
    pub fn add_environment(&mut self, key: &str, value: String) -> Result<(), String> {
        add_env(&mut self.env, key, value)
    }

    /// Add a role-scoped plain environment variable after validation.
    pub fn add_role_environment(
        &mut self,
        role: impl Into<RoleName>,
        key: &str,
        value: String,
    ) -> Result<(), String> {
        let role = role.into();
        add_env(self.role_env.entry(role).or_default(), key, value)
    }

    /// Number of configured role overrides.
    pub fn configured_role_count(&self) -> usize {
        self.role_env.values().filter(|env| !env.is_empty()).count()
    }

    /// Number of workspace and role-scoped environment variables.
    pub fn environment_count(&self) -> usize {
        self.env.len() + self.role_env.values().map(Vec::len).sum::<usize>()
    }
}

fn add_env(target: &mut Vec<EnvVar>, key: &str, value: String) -> Result<(), String> {
    let key = key.trim();
    if let Some(error) = env_key_error(key) {
        wipe_string_owned(value);
        return Err(error);
    }
    if target.iter().any(|env| env.key == key) {
        wipe_string_owned(value);
        return Err(format!("{key} is already configured"));
    }
    target.push(EnvVar {
        key: key.to_owned(),
        value: EnvValue::Plain(value),
    });
    Ok(())
}

fn wipe_string_owned(value: String) {
    let mut secret = termrock::Secret::new(value);
    secret.zeroize();
}

// ─────────────────────────────────────────────────────────────────────────────
// UI Composition: EditorScreen
// ─────────────────────────────────────────────────────────────────────────────

/// Reusable Workspace Editor screen composition.
pub struct EditorScreen;

impl EditorScreen {
    /// Draw the Editor tab strip, active tab content, and actions footer.
    pub fn draw(ui: &mut Ui<'_>, area: Rect, editor: &EditorState, world: &World, focused: bool) {
        let full = ui.full();

        // 1. Draw tabs strip
        let tab_items = Tab::ALL;
        let mut tab_state = TabsState::default();
        tab_state.set_active(
            usize::from(editor.tab.alias().saturating_sub(1)),
            ItemKey::num(u64::from(editor.tab.alias())),
        );

        let tabs = Tabs::new(TABS)
            .key(|tab: &Tab| ItemKey::num(u64::from(tab.alias())))
            .row(|tab: &Tab, row| {
                let name = match tab {
                    Tab::General => "General",
                    Tab::Mounts => "Mounts",
                    Tab::Roles => "Roles",
                    Tab::Environments => "Environments",
                    Tab::Accounts => "Accounts",
                };
                if editor.dirty && *tab == editor.tab {
                    row.label_spans(&[
                        Span::new(name),
                        Span::new(" "),
                        Span::new("•").role(Role::Warning),
                    ]);
                } else {
                    row.label(name);
                }
            });

        tabs.draw(
            ui,
            Rect::new(2, 3, full.width.saturating_sub(4), 2),
            &tab_state,
            &tab_items,
        );

        // 2. Draw tab content inside the tab body below the strip.
        let tab_body = Rect::new(
            area.x.saturating_add(1),
            area.y.saturating_add(4),
            area.width.saturating_sub(2),
            area.height.saturating_sub(6),
        );
        match editor.tab {
            Tab::General => Self::draw_general(ui, tab_body, editor, focused),
            Tab::Mounts => Self::draw_mounts(ui, tab_body, editor, world, focused),
            Tab::Roles => Self::draw_roles(ui, tab_body, editor, world, focused),
            Tab::Environments => Self::draw_environments(ui, tab_body, editor, world, focused),
            Tab::Accounts => Self::draw_accounts(ui, tab_body, editor, world, focused),
        }

        // 3. Draw bottom action buttons (Cancel and Save…), right-anchored
        // like the tag `row_layout_right` pair.
        let y = area.bottom().saturating_sub(1);
        let right = area.x.saturating_add(area.width.saturating_sub(4));
        let cancel_x = right.saturating_sub(18);
        let save_disabled =
            editor.saving.is_some() || (editor.change_count() == 0 && !editor.is_create());
        Button::new(FORM.sub("cancel"), "Cancel")
            .variant(Variant::SUBTLE)
            .draw(ui, Rect::new(cancel_x, y, 8, 1));
        Button::new(SAVE, "Save…")
            .variant(Variant::PRIMARY)
            .disabled(save_disabled)
            .draw(ui, Rect::new(cancel_x.saturating_add(11), y, 7, 1));
    }

    /// Required-field marker in accent, matching the tag name field.
    const NAME_STAR: [(Part, StylePatch); 1] =
        [(Part::MARKER, StylePatch::new().set_fg(Role::Accent))];

    fn draw_general(ui: &mut Ui<'_>, area: Rect, editor: &EditorState, _focused: bool) {
        let x = area.x.saturating_add(2);
        let fw = area.width.saturating_sub(4).min(72);
        // Name field: controlled value on an idle state. A `begin` here
        // would force the EDITING underline the tag only shows mid-edit.
        let name_state = TextInputState::default();
        Field::new("Name", TextInput::new(NAME).value(&editor.pending.name))
            .required(true)
            .help("Directory basename by default")
            .patch_part(&Self::NAME_STAR)
            .draw(ui, Rect::new(x, area.y, fw, 3), &name_state);

        // Working directory: bare rows, value padded to the tag width.
        let vw = fw.saturating_sub(14);
        let workdir_value = format!(
            "{:<w$}",
            truncate(&editor.pending.workdir, vw),
            w = vw as usize
        );
        let workdir_state = ListState::default();
        let workdir_items = [
            ("Working directory *", false),
            (workdir_value.as_str(), true),
            ("Inside the Construct", false),
        ];
        List::new(FORM.sub("workdir-list"))
            .bare(true)
            .row(|&(text, is_val): &(&str, bool), row| {
                if is_val {
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    row.label_patched(text, &p);
                } else if text.ends_with('*') {
                    let p = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Secondary))
                        .add(Modifier::BOLD);
                    row.label_patched(text, &p);
                } else {
                    let p = StylePatch::new().set_fg(Role::BorderStrong);
                    row.label_patched(text, &p);
                }
            })
            .draw(
                ui,
                Rect::new(x, area.y.saturating_add(4), fw, 3),
                &workdir_state,
                &workdir_items,
            );

        // Checkboxes
        Checkbox::new(KEEP_AWAKE, "Keep awake")
            .checked(editor.pending.keep_awake)
            .draw(ui, Rect::new(x, area.y.saturating_add(8), 30, 1));

        let badge_state = ListState::default();
        List::new(FORM.sub("macos-badge"))
            .bare(true)
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::BorderStrong);
                row.label_patched("macOS only", &p);
            })
            .draw(
                ui,
                Rect::new(x.saturating_add(30), area.y.saturating_add(8), 15, 1),
                &badge_state,
                &[()],
            );

        Checkbox::new(GIT_PULL, "Git pull before launch")
            .checked(editor.pending.git_pull)
            .draw(ui, Rect::new(x, area.y.saturating_add(9), 30, 1));

        // On dirty exit
        let dirty_exit_label_state = ListState::default();
        List::new(FORM.sub("dirty-exit-label"))
            .bare(true)
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                row.label_patched("On dirty exit", &p);
            })
            .draw(
                ui,
                Rect::new(x.saturating_add(2), area.y.saturating_add(11), 20, 1),
                &dirty_exit_label_state,
                &[()],
            );

        let mut select_state = SelectState::default();
        let policy_index = match editor.pending.dirty_policy {
            DirtyExitPolicy::Ask => 0,
            DirtyExitPolicy::Keep => 1,
            DirtyExitPolicy::Discard => 2,
        };
        select_state.set_value(Some(ItemKey::index(policy_index)));
        Select::new(FORM.sub("on-dirty-exit")).draw(
            ui,
            Rect::new(x, area.y.saturating_add(12), fw.min(48), 1),
            &select_state,
            &[
                "ask · show the exit dialog",
                "keep · preserve changes silently",
                "discard · drop changes silently",
            ],
        );

        // Drawn last so the focus ring visits the workdir row before the
        // picker button (TABS → NAME → WORKDIR → KEEP_AWAKE).
        Button::new(FORM.sub("choose"), "Choose…").draw(
            ui,
            Rect::new(
                x.saturating_add(fw.saturating_sub(11)),
                area.y.saturating_add(5),
                11,
                1,
            ),
        );
    }

    /// Tag `column_widths(Mounts, Workspace, avail)`: Destination, Mode,
    /// Isolation, Kind, Source. Zero widths hide the column.
    fn mount_widths(avail: u16) -> [u16; 5] {
        let fixed = 4 + 9 + 6;
        if avail >= 90 {
            let rest = avail.saturating_sub(fixed + 8);
            let dest = (rest * 45 / 100).max(20);
            [dest, 4, 9, 6, rest.saturating_sub(dest)]
        } else if avail >= 60 {
            [avail.saturating_sub(fixed + 6), 4, 9, 6, 0]
        } else {
            [avail.saturating_sub(4 + 9 + 4), 4, 9, 0, 0]
        }
    }

    fn draw_mounts(
        ui: &mut Ui<'_>,
        area: Rect,
        editor: &EditorState,
        world: &World,
        _focused: bool,
    ) {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Change {
            None,
            Added,
            Modified,
        }

        #[derive(Clone)]
        enum MountRow {
            Mount {
                dest: String,
                mode: &'static str,
                isolation: String,
                kind: &'static str,
                source: String,
                change: Change,
                selected: bool,
            },
            Add,
        }

        let widths = Self::mount_widths(area.width.saturating_sub(7));
        let hidden = widths.iter().filter(|w| **w == 0).count();

        // Header: bare row, truncated (never padded) muted titles.
        let header_state = ListState::default();
        let header = ["Destination", "Mode", "Isolation", "Kind", "Source"];
        let mut htracks: Vec<Track> = vec![];
        let mut htexts: Vec<String> = vec![];
        for (i, h) in header.iter().enumerate() {
            if widths[i] == 0 {
                continue;
            }
            htexts.push(truncate(h, widths[i]));
            htracks.push(Track::Fixed(widths[i]));
        }
        List::new(FORM.sub("mounts-header"))
            .bare(true)
            .row(|_, row| {
                let mut cols = row.columns_with_gap(&htracks, 2);
                let p = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                for (i, t) in htexts.iter().enumerate() {
                    cols.cell(i).patch(&p).text(t);
                }
            })
            .draw(
                ui,
                Rect::new(
                    area.x.saturating_add(6),
                    area.y,
                    area.width.saturating_sub(7),
                    1,
                ),
                &header_state,
                &[()],
            );
        if hidden > 0 {
            let tag_state = ListState::default();
            let tag = format!("{hidden}›");
            List::new(FORM.sub("mounts-hidden"))
                .bare(true)
                .row(|_, row| {
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Faint));
                    row.label_patched(&tag, &p);
                })
                .draw(
                    ui,
                    Rect::new(area.right().saturating_sub(2), area.y, 2, 1),
                    &tag_state,
                    &[()],
                );
        }

        // Rows from the pending draft compared against the original.
        let mut rows: Vec<MountRow> = vec![];
        for (i, m) in editor.pending.mounts.iter().enumerate() {
            let change = match editor
                .original
                .as_ref()
                .and_then(|o| o.mounts.iter().find(|o| o.destination == m.destination))
            {
                None => Change::Added,
                Some(o) if o != m => Change::Modified,
                _ => Change::None,
            };
            rows.push(MountRow::Mount {
                dest: m.destination.clone(),
                mode: m.mode_label(),
                isolation: m.isolation.label().to_lowercase(),
                kind: if matches!(m.source, MountSource::Git(_)) {
                    "git"
                } else {
                    "host"
                },
                source: world.tilde(m.source_label()),
                change,
                selected: i == 0,
            });
        }
        rows.push(MountRow::Add);

        // Change glyph plus the shown data columns; exact fit at every size.
        let mut tracks: Vec<Track> = vec![Track::Fixed(1)];
        for w in widths.iter().filter(|w| **w > 0) {
            tracks.push(Track::Fixed(*w));
        }
        // Shown data-column index per track (track 0 is the change glyph).
        let mut shown: Vec<usize> = vec![];
        for (i, w) in widths.iter().enumerate() {
            if *w > 0 {
                shown.push(i);
            }
        }
        let mut state = ListState::default();
        state.set_cursor(0, ItemKey::index(0));
        state.choose(Some(ItemKey::index(0)));
        List::new(MOUNTS_LIST)
            .bare_item(&|item: &MountRow| match item {
                MountRow::Mount { selected, .. } => !selected,
                MountRow::Add => true,
            })
            .row(|item, row| {
                // Plain rows voice the tag lead the stock marker cell
                // leaves on the container: black gutter, secondary blank.
                let bare_row = match item {
                    MountRow::Mount { selected, .. } => !selected,
                    MountRow::Add => true,
                };
                if bare_row {
                    row.label_spans(&[
                        Span::new(" ").role(Role::CurrentSurface),
                        Span::new(" ").role(Role::Fg(FgStep::Secondary)),
                        Span::new(" "),
                    ]);
                }
                let mut cols = row.columns_with_gap(&tracks, 2);
                match item {
                    MountRow::Mount {
                        dest,
                        mode,
                        isolation,
                        kind,
                        source,
                        change,
                        selected,
                    } => {
                        // Tag `row()`: bold only where the keyboard is.
                        let bold = if *selected && editor.focus == EditorFocus::Body {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        };
                        let (glyph, role) = match change {
                            Change::Modified => ("•", Role::Warning),
                            Change::Added => ("+", Role::Fg(FgStep::Primary)),
                            Change::None => (" ", Role::Fg(FgStep::Primary)),
                        };
                        let p = StylePatch::new().set_fg(role).add(bold);
                        cols.cell(0).patch(&p).text(glyph);
                        let cells: [&str; 5] = [dest, mode, isolation, kind, source];
                        for (ti, ci) in shown.iter().enumerate() {
                            let cw = widths[*ci] as usize;
                            let text = format!("{:<w$}", truncate(cells[*ci], widths[*ci]), w = cw);
                            let role = match *ci {
                                0 => Role::Fg(FgStep::Primary),
                                1 | 2 => Role::Fg(FgStep::Secondary),
                                _ => Role::Fg(FgStep::Muted),
                            };
                            let p = StylePatch::new().set_fg(role).add(bold);
                            cols.cell(ti.saturating_add(1)).patch(&p).text(&text);
                        }
                    }
                    MountRow::Add => {
                        let p = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
                        cols.cell(0).patch(&p).text(" ");
                        let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                        cols.cell(1).patch(&p).text("+ Add mount");
                    }
                }
            })
            .draw(
                ui,
                Rect::new(
                    area.x,
                    area.y.saturating_add(1),
                    area.width,
                    rows.len().saturating_add(1) as u16,
                ),
                &state,
                &rows,
            );
    }

    fn draw_roles(
        ui: &mut Ui<'_>,
        area: Rect,
        editor: &EditorState,
        world: &World,
        _focused: bool,
    ) {
        #[derive(Clone)]
        enum RoleRow {
            Role {
                name: String,
                allowed: bool,
                is_default: bool,
                changed: bool,
                meta: String,
                tone: Role,
                selected: bool,
            },
            Load,
        }

        // Policy entries carry `namespace/name` (fixture canonical form);
        // the tab compares and displays short names like the tag.
        fn short_role(name: &str) -> &str {
            name.rsplit('/').next().unwrap_or(name)
        }
        fn allows(policy: &RolePolicy, name: &str) -> bool {
            match &policy.allowed {
                AllowedRoles::All => true,
                AllowedRoles::Custom(list) => list.iter().any(|r| short_role(r) == name),
            }
        }
        // Rows mirror tag `build_roles`: world order, policy extras,
        // then the loader row.
        let mut names: Vec<String> = world.roles.iter().map(|r| r.name.clone()).collect();
        if let AllowedRoles::Custom(list) = &editor.pending.roles.allowed {
            for r in list {
                if !names.iter().any(|n| n == short_role(r)) {
                    names.push(short_role(r).to_owned());
                }
            }
        }
        if let Some(d) = &editor.pending.roles.default
            && !names.iter().any(|n| n == short_role(d))
        {
            names.push(short_role(d).to_owned());
        }
        let name_w = names
            .iter()
            .map(|n| width(n))
            .max()
            .unwrap_or(8)
            .clamp(8, 24);
        let list_focused = editor.focus == EditorFocus::Body;
        let mut rows: Vec<RoleRow> = vec![];
        for (i, name) in names.iter().enumerate() {
            let entry = world.roles.iter().find(|r| &r.name == name);
            let allowed = allows(&editor.pending.roles, name);
            let is_default = editor
                .pending
                .roles
                .default
                .as_deref()
                .is_some_and(|d| short_role(d) == name.as_str());
            let changed = editor.original.as_ref().is_some_and(|o| {
                allows(&o.roles, name) != allowed
                    || o.roles
                        .default
                        .as_deref()
                        .is_some_and(|d| short_role(d) == name.as_str())
                        != is_default
            });
            let (meta, tone) = match entry {
                Some(e) if e.load_error.is_some() => (
                    format!(
                        "! load error · {}",
                        e.load_error.as_deref().unwrap_or_default()
                    ),
                    Role::Danger,
                ),
                Some(e) if !e.in_registry => (
                    format!("{} · not in registry", e.source.label()),
                    Role::Warning,
                ),
                Some(e) => (
                    format!(
                        "registry · {} · {}",
                        if e.trusted { "trusted" } else { "untrusted" },
                        e.description
                    ),
                    Role::Fg(FgStep::Muted),
                ),
                None => ("not in registry".into(), Role::Warning),
            };
            rows.push(RoleRow::Role {
                name: name.clone(),
                allowed,
                is_default,
                changed,
                meta,
                tone,
                selected: i == 0,
            });
        }
        rows.push(RoleRow::Load);

        // Header: bare row, title plus right-aligned default note.
        let (allowed_n, total) = match &editor.pending.roles.allowed {
            AllowedRoles::All => (world.roles.len(), world.roles.len()),
            AllowedRoles::Custom(list) => (list.len(), world.roles.len()),
        };
        let head = match editor.pending.roles.allowed {
            AllowedRoles::All => "Allowed roles  all".to_owned(),
            AllowedRoles::Custom(_) => format!("Allowed roles  {allowed_n} of {total}"),
        };
        let def = format!(
            "default {}",
            editor
                .pending
                .roles
                .default
                .as_deref()
                .map(|d| format!("★ {}", short_role(d)))
                .unwrap_or("none".into())
        );
        let header_state = ListState::default();
        List::new(FORM.sub("roles-header"))
            .bare(true)
            .row(|_, row| {
                let head_w = width(&head);
                let mw = width(&def);
                let def_x = area.right().saturating_sub(mw.saturating_add(2));
                let title_x = area.x.saturating_add(2);
                let pad = def_x.saturating_sub(title_x.saturating_add(head_w)) as usize;
                row.label_spans(&[
                    Span::new("  "),
                    Span::new(&head)
                        .role(Role::Fg(FgStep::Secondary))
                        .modifier(Modifier::BOLD),
                    Span::new(&" ".repeat(pad)),
                    Span::new(&def).role(Role::Fg(FgStep::Faint)),
                ]);
            })
            .draw(
                ui,
                Rect::new(area.x, area.y, area.width, 1),
                &header_state,
                &[()],
            );

        // Rows: change, mark, fitted name, star, truncated meta.
        let viewport_h = area.height.saturating_sub(1);
        let has_sb = rows.len() > viewport_h as usize;
        let row_right = area.right().saturating_sub(u16::from(has_sb));
        let mut state = ListState::default();
        state.set_cursor(0, ItemKey::index(0));
        state.choose(Some(ItemKey::index(0)));
        List::new(ROLES_LIST)
            .bare_item(&|item: &RoleRow| !matches!(item, RoleRow::Role { selected: true, .. }))
            .row(|item, row| {
                let row_focused =
                    matches!(item, RoleRow::Role { selected: true, .. }) && list_focused;
                let bold = if row_focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                };
                if !matches!(item, RoleRow::Role { selected: true, .. }) {
                    row.label_spans(&[
                        Span::new(" ").role(Role::CurrentSurface),
                        Span::new(" ").role(Role::Fg(FgStep::Secondary)),
                        Span::new(" "),
                    ]);
                }
                match item {
                    RoleRow::Load => {
                        let role = if row_focused {
                            Role::Fg(FgStep::Primary)
                        } else {
                            Role::Fg(FgStep::Secondary)
                        };
                        row.label_spans(&[Span::new(" "), Span::new("+ Load role…").role(role)]);
                    }
                    RoleRow::Role {
                        name,
                        allowed,
                        is_default,
                        changed,
                        meta,
                        tone,
                        selected,
                    } => {
                        let name_bold = if *selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        };
                        let fitted = format!("{:<w$}", name, w = name_w as usize);
                        let sx = 10 + name_w;
                        let mx = sx.saturating_add(3);
                        let content_right = row_right.saturating_sub(area.x);
                        let meta_w = content_right.saturating_sub(mx.saturating_add(1)) as usize;
                        let star_role = if row_focused {
                            Span::new(if *is_default { "★" } else { " " })
                                .role(Role::Fg(FgStep::Primary))
                                .modifier(Modifier::BOLD)
                        } else {
                            Span::new(if *is_default { "★" } else { " " })
                                .role(Role::Fg(FgStep::Secondary))
                        };
                        row.label_spans(&[
                            Span::new(if *changed { "•" } else { " " })
                                .role(Role::Warning)
                                .modifier(bold),
                            Span::new(" "),
                            Span::new(if *allowed { "[✓]" } else { "[ ]" })
                                .role(if *allowed {
                                    Role::Fg(FgStep::Primary)
                                } else {
                                    Role::Fg(FgStep::Muted)
                                })
                                .modifier(bold),
                            Span::new(" "),
                            Span::new(&fitted)
                                .role(Role::Fg(FgStep::Primary))
                                .modifier(name_bold),
                            Span::new(" "),
                            star_role,
                            Span::new("  "),
                            Span::new(&truncate(meta, meta_w as u16))
                                .role(*tone)
                                .modifier(bold),
                        ]);
                    }
                }
            })
            .draw(
                ui,
                Rect::new(area.x, area.y.saturating_add(1), area.width, viewport_h),
                &state,
                &rows,
            );
    }

    /// Tag `column_widths(Environments, Workspace, avail)`: Key, Value,
    /// Source. Always fully visible.
    fn env_widths(avail: u16) -> [u16; 3] {
        let key = 18.min(avail / 3);
        let value = if avail >= 90 { 24 } else { 18 };
        [key, value, avail.saturating_sub(key + value + 4)]
    }

    /// Masked value and source cells for an environment variable.
    fn env_cells(var: &EnvVar) -> (String, String) {
        match &var.value {
            EnvValue::Plain(v) => (mask(v), "plain".into()),
            EnvValue::OnePassword(r) => ("*".repeat(16), format!("[op] {}", r.display_path())),
            EnvValue::HostEnv(name) => (format!("${name}"), "host env".into()),
        }
    }

    fn draw_environments(
        ui: &mut Ui<'_>,
        area: Rect,
        editor: &EditorState,
        world: &World,
        _focused: bool,
    ) {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Change {
            None,
            Added,
            Modified,
        }

        #[derive(Clone)]
        enum EnvRow {
            Section {
                title: String,
                meta: String,
                folded: Option<bool>,
            },
            Var {
                key: String,
                value: String,
                source: String,
                change: Change,
                selected: bool,
                scope: Option<String>,
            },
            Add {
                text: String,
            },
        }

        // Wide inspector column; the list keeps the remainder.
        // Tag `config.rs`: the inspector card splits off at 150+ columns.
        let viewport_w = area.width;
        let wide = viewport_w >= 150;
        let (list_area, card_area) = if wide {
            (
                Rect::new(area.x, area.y, area.width.saturating_sub(58), area.height),
                Some(Rect::new(
                    area.right().saturating_sub(56),
                    area.y,
                    56,
                    area.height.min(24),
                )),
            )
        } else {
            (area, None)
        };
        let widths = Self::env_widths(list_area.width.saturating_sub(7));

        // Rows mirror tag `env_rows`: workspace section, role summary,
        // one section per configured role, then the override loader.
        let configured: Vec<String> = editor.pending.role_env.keys().cloned().collect();
        let registry: Vec<&str> = world.roles.iter().map(|r| r.name.as_str()).collect();
        let orig_env = editor
            .original
            .as_ref()
            .map(|o| o.env.clone())
            .unwrap_or_default();
        let mut rows: Vec<EnvRow> = vec![];
        rows.push(EnvRow::Section {
            title: "Workspace".into(),
            meta: format!(
                "{} {}",
                editor.pending.env.len(),
                if editor.pending.env.len() == 1 {
                    "var"
                } else {
                    "vars"
                }
            ),
            folded: None,
        });
        for (i, e) in editor.pending.env.iter().enumerate() {
            let change = match orig_env.iter().find(|o| o.key == e.key) {
                None => Change::Added,
                Some(o) if o != e => Change::Modified,
                _ => Change::None,
            };
            let (value, source) = Self::env_cells(e);
            rows.push(EnvRow::Var {
                key: e.key.clone(),
                value,
                source,
                change,
                selected: i == 0,
                scope: None,
            });
        }
        rows.push(EnvRow::Add {
            text: "+ Add environment variable".into(),
        });
        rows.push(EnvRow::Section {
            title: "Role overrides".into(),
            meta: if configured.is_empty() {
                format!("none · {} in the registry", registry.len())
            } else {
                format!(
                    "{} configured · {} in the registry",
                    configured.len(),
                    registry.len()
                )
            },
            folded: None,
        });
        // Role keys carry `namespace/name`; sections compare and display
        // short names like the tag (tag `config.rs` roles are short).
        fn short_role(name: &str) -> &str {
            name.rsplit('/').next().unwrap_or(name)
        }
        for r in &configured {
            let short = short_role(r);
            let empty = vec![];
            let vars = editor.pending.role_env.get(r).unwrap_or(&empty);
            let mut meta = format!(
                "{} {}",
                vars.len(),
                if vars.len() == 1 { "var" } else { "vars" }
            );
            if !registry.iter().any(|x| short_role(x) == short) {
                meta = format!("not in registry · {meta}");
            }
            rows.push(EnvRow::Section {
                title: format!("Role: {short}"),
                meta,
                folded: Some(false),
            });
            let orig_role = editor
                .original
                .as_ref()
                .and_then(|o| o.role_env.get(r))
                .cloned()
                .unwrap_or_default();
            for e in vars {
                let change = match orig_role.iter().find(|o| o.key == e.key) {
                    None => Change::Added,
                    Some(o) if o != e => Change::Modified,
                    _ => Change::None,
                };
                let (value, source) = Self::env_cells(e);
                rows.push(EnvRow::Var {
                    key: e.key.clone(),
                    value,
                    source,
                    change,
                    selected: false,
                    scope: Some(r.clone()),
                });
            }
            rows.push(EnvRow::Add {
                text: format!("+ Add {short} environment variable"),
            });
        }
        rows.push(EnvRow::Add {
            text: "+ Add role override…".into(),
        });

        let list_focused = editor.focus == EditorFocus::Body;
        let cursor_idx = rows
            .iter()
            .position(|r| matches!(r, EnvRow::Var { .. }))
            .unwrap_or(1);
        let mut state = ListState::default();
        state.set_cursor(cursor_idx, ItemKey::index(cursor_idx));
        state.choose(Some(ItemKey::index(cursor_idx)));
        let tracks = [
            Track::Fixed(1),
            Track::Fixed(widths[0]),
            Track::Fixed(widths[1]),
            Track::Fixed(widths[2]),
        ];
        // Tag `config.rs`: the content rect excludes the scrollbar column
        // only while the list overflows (`row_w`).
        let viewport_h = list_area.height;
        let has_sb = rows.len() > viewport_h as usize;
        let row_w = list_area.width.saturating_sub(u16::from(has_sb));
        List::new(ENV_LIST)
            // Every row is bare: the stock non-bare row reserves a right
            // pad column that would steal the source cell's last column.
            // The chosen row still gets the selected tint fill; the cursor
            // gutter and marker are voiced manually below like the tag.
            .bare(true)
            .row(|item, row| {
                let is_cursor = matches!(item, EnvRow::Var { selected: true, .. });
                let row_focused = is_cursor && list_focused;
                let bold = if row_focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                };
                match item {
                    EnvRow::Section {
                        title,
                        meta,
                        folded,
                    } => {
                        let title_w = width(title);
                        let mw = width(meta);
                        let mut spans = vec![
                            Span::new("  "),
                            match folded {
                                Some(f) => Span::new(if *f { "▸" } else { "▾" })
                                    .role(Role::Fg(FgStep::Secondary)),
                                None => Span::new("  "),
                            },
                        ];
                        if folded.is_some() {
                            spans.push(Span::new(" "));
                        }
                        spans.push(
                            Span::new(title)
                                .role(Role::Fg(FgStep::Secondary))
                                .modifier(Modifier::BOLD),
                        );
                        let has_meta = mw > 0 && row_w > mw + title_w + 8;
                        let pad = if has_meta {
                            let meta_x = row_w.saturating_sub(mw.saturating_add(1));
                            meta_x.saturating_sub(title_w.saturating_add(4)) as usize
                        } else {
                            0
                        };
                        let pad_str = " ".repeat(pad);
                        if has_meta {
                            spans.push(Span::new(&pad_str));
                            spans.push(Span::new(meta).role(
                                if meta.starts_with("not in registry") {
                                    Role::Warning
                                } else {
                                    Role::Fg(FgStep::Faint)
                                },
                            ));
                        }
                        row.label_spans(&spans);
                    }
                    EnvRow::Var {
                        key,
                        value,
                        source,
                        change,
                        selected,
                        ..
                    } => {
                        if is_cursor {
                            // Tag cursor lead: accent gutter + marker where
                            // the keyboard is, secondary blanks otherwise.
                            let marker = if row_focused {
                                Role::Accent
                            } else {
                                Role::Fg(FgStep::Secondary)
                            };
                            row.label_spans(&[
                                Span::new("▎").role(marker).modifier(bold),
                                Span::new("›").role(marker).modifier(bold),
                                Span::new(" ")
                                    .role(Role::Fg(FgStep::Primary))
                                    .modifier(bold),
                            ]);
                        } else {
                            row.label_spans(&[
                                Span::new(" ").role(Role::CurrentSurface),
                                Span::new(" ").role(Role::Fg(FgStep::Secondary)),
                                Span::new(" "),
                            ]);
                        }
                        let key_bold = if *selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        };
                        let (glyph, role) = match change {
                            Change::Modified => ("•", Role::Warning),
                            Change::Added => ("+", Role::Fg(FgStep::Primary)),
                            Change::None => (" ", Role::Fg(FgStep::Primary)),
                        };
                        let mut cols = row.columns_with_gap(&tracks, 2);
                        let p = StylePatch::new().set_fg(role).add(bold);
                        cols.cell(0).patch(&p).text(glyph);
                        let cells = [key, value, source];
                        for (ci, cw) in widths.iter().enumerate() {
                            let text =
                                format!("{:<w$}", truncate(cells[ci], *cw), w = *cw as usize);
                            let role = match ci {
                                0 => Role::Fg(FgStep::Primary),
                                1 => Role::Fg(FgStep::Secondary),
                                _ => Role::Fg(FgStep::Muted),
                            };
                            let b = if ci == 0 { key_bold } else { bold };
                            let p = StylePatch::new().set_fg(role).add(b);
                            cols.cell(ci.saturating_add(1)).patch(&p).text(&text);
                        }
                    }
                    EnvRow::Add { text } => {
                        // Tag add rows start their text at `rect.x + 6`.
                        row.label_spans(&[
                            Span::new(" ").role(Role::CurrentSurface),
                            Span::new(" ").role(Role::Fg(FgStep::Secondary)),
                            Span::new("    "),
                            Span::new(text).role(Role::Fg(FgStep::Secondary)),
                        ]);
                    }
                }
            })
            .draw(
                ui,
                Rect::new(list_area.x, list_area.y, list_area.width, list_area.height),
                &state,
                &rows,
            );

        // Inspector card for the cursor variable on wide screens.
        if let Some(card) = card_area
            && let Some(EnvRow::Var { key, scope, .. }) = rows.get(cursor_idx)
        {
            let role = scope.clone();
            let vars = match &role {
                Some(r) => editor.pending.role_env.get(r).cloned().unwrap_or_default(),
                None => editor.pending.env.clone(),
            };
            let orig_vars = match &role {
                Some(r) => editor
                    .original
                    .as_ref()
                    .and_then(|o| o.role_env.get(r))
                    .cloned()
                    .unwrap_or_default(),
                None => orig_env,
            };
            let p = vars.iter().find(|e| &e.key == key);
            let o = orig_vars.iter().find(|e| &e.key == key);
            let mut lines: Vec<(String, String, Role)> = vec![
                (
                    "Pending".into(),
                    p.map(|e| e.value.source_label().to_owned())
                        .unwrap_or("removed".into()),
                    Role::Fg(FgStep::Primary),
                ),
                (
                    "Original".into(),
                    o.map(|e| e.value.source_label().to_owned())
                        .unwrap_or("new".into()),
                    Role::Fg(FgStep::Muted),
                ),
            ];
            if let Some(EnvValue::OnePassword(r)) = p.map(|e| &e.value) {
                lines.push(("Reference".into(), r.canonical(), Role::Fg(FgStep::Muted)));
                lines.push((
                    "Vault".into(),
                    format!("{} · {}", r.account, r.vault_name),
                    Role::Fg(FgStep::Muted),
                ));
            }
            lines.push((
                "Resolution".into(),
                "resolved at launch · never stored in the Construct image".into(),
                Role::Fg(FgStep::Faint),
            ));
            let title = format!("Variable · {key}");
            let meta = role
                .as_deref()
                .map(|r| format!("role {}", short_role(r)))
                .unwrap_or("workspace".into());
            Panel::new(FORM.sub("env-card"))
                .title(&title)
                .meta(&meta)
                .draw(ui, card, |ui, inner| {
                    let card_state = ListState::default();
                    List::new(FORM.sub("env-card-rows"))
                        .bare(true)
                        .row(|line: &(String, String, Role), row| {
                            let mut cols =
                                row.columns_with_gap(&[Track::Fixed(14), Track::Flex(1)], 0);
                            let p = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                            cols.cell(0).patch(&p).text(&truncate(&line.0, 14));
                            let vw = inner.width.saturating_sub(14);
                            let p = StylePatch::new().set_fg(line.2);
                            cols.cell(1).patch(&p).text(&truncate(&line.1, vw));
                        })
                        .draw(ui, inner, &card_state, &lines);
                });
        }
    }

    fn draw_accounts(
        ui: &mut Ui<'_>,
        area: Rect,
        editor: &EditorState,
        world: &World,
        _focused: bool,
    ) {
        #[derive(Clone)]
        enum AccRow {
            Provider { label: String, active: usize },
            Account { id: AccountId, selected: bool },
        }

        // Tag `editor.rs render_accounts`: a head line, provider headings
        // with right-aligned activity meta, one row per registry account.
        let registry = &world.accounts;
        let effective = editor.pending.effective_accounts(registry);
        let inherited = effective
            .iter()
            .filter(|e| e.origin == Effective::InheritedDefault)
            .count();
        let enabled_here = effective.len().saturating_sub(inherited);
        let head = format!(
            "Active accounts  {} effective · {inherited} inherited · {enabled_here} enabled here",
            effective.len()
        );
        let hint = "registry in Accounts (c)";
        let tab_w = area.width;
        let head_w = width(&head);
        let hint_w = width(hint);
        let show_hint = tab_w > head_w + hint_w + 8;
        let head_text = truncate(&head, area.width.saturating_sub(4));
        let head_pad = area
            .width
            .saturating_sub(width(hint) as u16 + 2)
            .saturating_sub(width(&head_text) as u16 + 2) as usize;
        let head_pad_str = " ".repeat(head_pad);
        {
            let head_state = ListState::default();
            List::new(FORM.sub("accounts-head"))
                .bare(true)
                .row(|_: &(), row| {
                    let mut spans = vec![
                        Span::new("  "),
                        Span::new(&head_text)
                            .role(Role::Fg(FgStep::Secondary))
                            .modifier(Modifier::BOLD),
                    ];
                    if show_hint {
                        spans.push(Span::new(&head_pad_str));
                        spans.push(Span::new(hint).role(Role::Fg(FgStep::Faint)));
                    }
                    row.label_spans(&spans);
                })
                .draw(
                    ui,
                    Rect::new(area.x, area.y, area.width, 1),
                    &head_state,
                    &[()],
                );
        }

        let body = Rect::new(
            area.x,
            area.y.saturating_add(1),
            area.width,
            area.height.saturating_sub(1),
        );
        let mut providers: Vec<Provider> = registry.accounts.iter().map(|a| a.provider).collect();
        providers.sort();
        providers.dedup();
        let mut rows: Vec<AccRow> = vec![];
        for p in providers {
            let ids: Vec<AccountId> = registry
                .sorted()
                .into_iter()
                .filter(|a| a.provider == p)
                .map(|a| a.id.clone())
                .collect();
            if ids.is_empty() {
                continue;
            }
            let active = effective.iter().filter(|e| e.provider == p).count();
            rows.push(AccRow::Provider {
                label: p.label().to_owned(),
                active,
            });
            rows.extend(ids.into_iter().map(|id| AccRow::Account {
                id,
                selected: false,
            }));
        }
        let mut cursor = 0usize;
        while matches!(rows.get(cursor), Some(AccRow::Provider { .. })) && cursor + 1 < rows.len() {
            cursor += 1;
        }
        if let Some(AccRow::Account { selected, .. }) = rows.get_mut(cursor) {
            *selected = true;
        }

        let name_w = registry
            .accounts
            .iter()
            .map(|a| width(&a.display_name))
            .max()
            .unwrap_or(8)
            .clamp(8, 28);
        let original_effective = editor
            .original
            .as_ref()
            .map(|o| o.effective_accounts(registry))
            .unwrap_or_default();
        let body_h = body.height;
        let has_sb = rows.len() > body_h as usize;
        let row_w = body.width.saturating_sub(u16::from(has_sb));
        let list_focused = editor.focus == EditorFocus::Body;

        let mut state = ListState::default();
        state.set_cursor(cursor, ItemKey::index(cursor));
        state.choose(Some(ItemKey::index(cursor)));
        List::new(ACCOUNTS_LIST)
            .bare(true)
            .row(|item, row| match item {
                AccRow::Provider { label, active } => {
                    let meta = match active {
                        0 => "none active".to_owned(),
                        1 => "1 active".to_owned(),
                        n => format!("{n} active · picker at session start"),
                    };
                    let mw = width(&meta) as u16;
                    let title_w = width(label) as u16;
                    let show_meta = row_w > mw + 20;
                    let pad = if show_meta {
                        row_w.saturating_sub(mw + 1).saturating_sub(title_w + 2) as usize
                    } else {
                        0
                    };
                    let pad_str = " ".repeat(pad);
                    let mut spans = vec![
                        Span::new("  "),
                        Span::new(label)
                            .role(Role::Fg(FgStep::Secondary))
                            .modifier(Modifier::BOLD),
                    ];
                    if show_meta {
                        spans.push(Span::new(&pad_str));
                        spans.push(Span::new(&meta).role(Role::Fg(FgStep::Faint)));
                    }
                    row.label_spans(&spans);
                }
                AccRow::Account { id, selected } => {
                    let Some(a) = registry.get(id) else {
                        return;
                    };
                    let eff = effective.iter().find(|e| &e.id == id);
                    let row_focused = *selected && list_focused;
                    let bold = if row_focused {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    };
                    let marker = if row_focused {
                        Role::Accent
                    } else {
                        Role::Fg(FgStep::Secondary)
                    };
                    let was_active = original_effective.iter().any(|e| &e.id == id);
                    let changed = was_active != eff.is_some()
                        || editor
                            .original
                            .as_ref()
                            .and_then(|o| o.accounts.preferred.get(&a.provider))
                            != editor.pending.accounts.preferred.get(&a.provider);
                    let active = eff.is_some();
                    let label = if a.origin == AccountOrigin::Discovered
                        && a.display_name == "discovered"
                    {
                        "host login".to_owned()
                    } else {
                        a.display_name.clone()
                    };
                    let label_trunc = truncate(&label, name_w);
                    let label_text = format!(
                        "{label_trunc}{}",
                        " ".repeat(name_w as usize - width(&label_trunc) as usize)
                    );
                    let origin = match eff.map(|e| e.origin) {
                        Some(o) => o.label().to_owned(),
                        None if a.default_for_provider => "disabled here".to_owned(),
                        None if a.origin == AccountOrigin::Discovered => {
                            "discovered on host".to_owned()
                        }
                        None => "available".to_owned(),
                    };
                    let origin_trunc = truncate(&origin, 22);
                    let origin_text = format!(
                        "{origin_trunc}{}",
                        " ".repeat(22usize.saturating_sub(width(&origin_trunc) as usize))
                    );
                    let status = usability_of(a).label();
                    let status_tone = if a.enabled && status == "ready" {
                        Role::Fg(FgStep::Muted)
                    } else {
                        Role::Warning
                    };
                    let status_text = truncate(&status, row_w.saturating_sub(name_w + 38));
                    let label_bold = if !a.enabled {
                        bold
                    } else if *selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    };
                    row.label_spans(&[
                        Span::new(if *selected { "▎" } else { " " })
                            .role(if *selected {
                                marker
                            } else {
                                Role::CurrentSurface
                            })
                            .modifier(bold),
                        Span::new(if *selected { "›" } else { " " })
                            .role(if *selected {
                                marker
                            } else {
                                Role::Fg(FgStep::Secondary)
                            })
                            .modifier(bold),
                        Span::new(" "),
                        Span::new(if changed { "•" } else { " " })
                            .role(Role::Warning)
                            .modifier(bold),
                        Span::new(" "),
                        Span::new(if active { "[✓]" } else { "[ ]" })
                            .role(if active {
                                Role::Fg(FgStep::Primary)
                            } else {
                                Role::Fg(FgStep::Muted)
                            })
                            .modifier(bold),
                        Span::new(" "),
                        Span::new(&label_text)
                            .role(if a.enabled {
                                Role::Fg(FgStep::Primary)
                            } else {
                                Role::Fg(FgStep::Faint)
                            })
                            .modifier(label_bold),
                        Span::new(" "),
                        Span::new(if eff.is_some_and(|e| e.preferred) {
                            "★"
                        } else {
                            " "
                        })
                        .role(if row_focused {
                            Role::Fg(FgStep::Primary)
                        } else {
                            Role::Fg(FgStep::Secondary)
                        })
                        .modifier(bold),
                        Span::new("  "),
                        Span::new(&origin_text)
                            .role(Role::Fg(FgStep::Muted))
                            .modifier(bold),
                        Span::new("  "),
                        Span::new(&status_text).role(status_tone).modifier(bold),
                    ]);
                }
            })
            .draw(ui, body, &state, &rows);
    }

    /// Draw the save preview modal dialog over the editor screen.
    pub fn draw_save_preview(ui: &mut Ui<'_>, _area: Rect, editor: &EditorState, _world: &World) {
        let modal_area = Rect::new(27, 14, 66, 13);
        Panel::new(CFG_FORM)
            .kind(PanelKind::Framed)
            .draw(ui, modal_area, |ui, body| {
                let list_state = ListState::default();
                let ws_name = editor.pending.name.as_str();
                let rows: [(&str, &str, bool, bool); 7] = [
                    ("Save workspace", "", false, true),
                    ("", "", false, false),
                    ("Workspace", ws_name, true, false),
                    (
                        "Scope",
                        "workspace config · ~/.jackin/workspaces/payments…",
                        true,
                        false,
                    ),
                    ("Changes", "1 change", true, false),
                    ("", "", false, false),
                    ("~ keep_awake true → false", "", false, false),
                ];

                List::new(CFG_FORM.sub("preview-rows"))
                    .row(
                        |&(label, val, is_kv, is_title): &(&str, &str, bool, bool), row| {
                            if is_title {
                                let p_bold = StylePatch::new()
                                    .set_fg(Role::Fg(FgStep::Primary))
                                    .add(Modifier::BOLD);
                                row.label_patched(&format!("  {label}"), &p_bold);
                            } else if is_kv {
                                let mut cols =
                                    row.columns_with_gap(&[Track::Fixed(11), Track::Flex(1)], 2);
                                let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                                cols.cell(0).patch(&p_muted).text(&format!("  {label}"));
                                cols.cell(1).text(val);
                            } else if !label.is_empty() {
                                let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                                row.label_patched(&format!("  {label}"), &p_sec);
                            }
                        },
                    )
                    .draw(
                        ui,
                        Rect::new(body.x, body.y.saturating_add(1), body.width, 8),
                        &list_state,
                        &rows,
                    );

                Button::new(PREVIEW_CANCEL, "Cancel").draw(
                    ui,
                    Rect::new(
                        body.right().saturating_sub(18),
                        body.bottom().saturating_sub(2),
                        8,
                        1,
                    ),
                );
                Button::new(SAVE, "Save").draw(
                    ui,
                    Rect::new(
                        body.right().saturating_sub(9),
                        body.bottom().saturating_sub(2),
                        6,
                        1,
                    ),
                );
            });
    }

    /// Render the dirty-exit dialog: Cancel/Discard/Save over the stay-or-leave question.
    pub fn draw_exit_dialog(ui: &mut Ui<'_>, _area: Rect, editor: &EditorState) {
        let changes = editor.change_count().max(1);
        let lost = if changes == 1 {
            "1 change would be lost.".to_owned()
        } else {
            format!("{changes} changes would be lost.")
        };
        let modal_area = Rect::new(33, 16, 54, 9);
        Panel::new(EXIT)
            .kind(PanelKind::Framed)
            .draw(ui, modal_area, |ui, body| {
                let list_state = ListState::default();
                let question = format!("Save changes before leaving? {lost}");
                let rows = ["Unsaved changes", "", question.as_str()];
                List::new(EXIT.sub("rows"))
                    .row(|label: &&str, row| {
                        if label.is_empty() {
                            return;
                        }
                        if *label == "Unsaved changes" {
                            let p_bold = StylePatch::new()
                                .set_fg(Role::Fg(FgStep::Primary))
                                .add(Modifier::BOLD);
                            row.label_patched(&format!("  {label}"), &p_bold);
                        } else {
                            let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                            row.label_patched(&format!("  {label}"), &p_sec);
                        }
                    })
                    .draw(
                        ui,
                        Rect::new(body.x, body.y, body.width, 3),
                        &list_state,
                        &rows,
                    );

                Button::new(EXIT_CANCEL, "Cancel").draw(
                    ui,
                    Rect::new(
                        body.right().saturating_sub(26),
                        body.bottom().saturating_sub(2),
                        8,
                        1,
                    ),
                );
                Button::new(EXIT_DISCARD, "Discard").draw(
                    ui,
                    Rect::new(
                        body.right().saturating_sub(17),
                        body.bottom().saturating_sub(2),
                        9,
                        1,
                    ),
                );
                Button::new(EXIT_SAVE, "Save").draw(
                    ui,
                    Rect::new(
                        body.right().saturating_sub(7),
                        body.bottom().saturating_sub(2),
                        6,
                        1,
                    ),
                );
            });
    }

    /// Editor hints for the bottom hint bar.
    pub fn hints(editor: &EditorState) -> HintLayer {
        if editor.preview_open || editor.exit_open {
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("← →"),
                        label: "Choose",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Label("Enter"),
                        label: "Confirm",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Label("Esc"),
                        label: "Cancel",
                        priority: 80,
                    },
                    Hint {
                        key: HintKey::Label("y / n"),
                        label: "Quick answer",
                        priority: 70,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            }
        } else if editor.focus == EditorFocus::Tabs {
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("← →"),
                        label: "Tab",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Label("1–5"),
                        label: "Jump",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Label("Enter"),
                        label: "Body",
                        priority: 80,
                    },
                    Hint {
                        key: HintKey::Label("[ ]"),
                        label: "Switch tab",
                        priority: 70,
                    },
                    Hint {
                        key: HintKey::Label("Ctrl+S"),
                        label: "Save",
                        priority: 60,
                    },
                    Hint {
                        key: HintKey::Label("Esc"),
                        label: "Back",
                        priority: 50,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            }
        } else {
            match editor.tab {
                Tab::General => HintLayer {
                    hints: vec![
                        Hint {
                            key: HintKey::Label("Enter"),
                            label: "Edit",
                            priority: 100,
                        },
                        Hint {
                            key: HintKey::Label("Space"),
                            label: "Toggle",
                            priority: 90,
                        },
                        Hint {
                            key: HintKey::Label("Tab"),
                            label: "Next",
                            priority: 80,
                        },
                        Hint {
                            key: HintKey::Label("[ ]"),
                            label: "Switch tab",
                            priority: 70,
                        },
                        Hint {
                            key: HintKey::Label("Ctrl+S"),
                            label: "Save",
                            priority: 60,
                        },
                        Hint {
                            key: HintKey::Label("Esc"),
                            label: "Back",
                            priority: 50,
                        },
                    ],
                    badge: None,
                    status: None,
                    centered: true,
                },
                Tab::Mounts => HintLayer {
                    hints: vec![
                        Hint {
                            key: HintKey::Label("Enter"),
                            label: "Edit…",
                            priority: 100,
                        },
                        Hint {
                            key: HintKey::Label("r"),
                            label: "Read-only",
                            priority: 90,
                        },
                        Hint {
                            key: HintKey::Label("i"),
                            label: "Isolation",
                            priority: 80,
                        },
                        Hint {
                            key: HintKey::Label("o"),
                            label: "Open source",
                            priority: 70,
                        },
                        Hint {
                            key: HintKey::Label("d"),
                            label: "Remove",
                            priority: 60,
                        },
                        Hint {
                            key: HintKey::Label("a"),
                            label: "Add mount…",
                            priority: 50,
                        },
                        Hint {
                            key: HintKey::Label("[ ]"),
                            label: "Switch tab",
                            priority: 40,
                        },
                        Hint {
                            key: HintKey::Label("Ctrl+S"),
                            label: "Save",
                            priority: 30,
                        },
                        Hint {
                            key: HintKey::Label("Esc"),
                            label: "Back",
                            priority: 20,
                        },
                    ],
                    badge: None,
                    status: None,
                    centered: true,
                },
                Tab::Roles => HintLayer {
                    hints: vec![
                        Hint {
                            key: HintKey::Label("Space"),
                            label: "Allow",
                            priority: 100,
                        },
                        Hint {
                            key: HintKey::Label("Enter"),
                            label: "Set default",
                            priority: 90,
                        },
                        Hint {
                            key: HintKey::Label("a"),
                            label: "Load role…",
                            priority: 80,
                        },
                        Hint {
                            key: HintKey::Label("/"),
                            label: "Filter",
                            priority: 70,
                        },
                        Hint {
                            key: HintKey::Label("[ ]"),
                            label: "Switch tab",
                            priority: 60,
                        },
                        Hint {
                            key: HintKey::Label("Ctrl+S"),
                            label: "Save",
                            priority: 50,
                        },
                        Hint {
                            key: HintKey::Label("Esc"),
                            label: "Back",
                            priority: 40,
                        },
                    ],
                    badge: None,
                    status: None,
                    centered: true,
                },
                Tab::Environments => HintLayer {
                    hints: vec![
                        Hint {
                            key: HintKey::Label("Enter"),
                            label: "Edit",
                            priority: 100,
                        },
                        Hint {
                            key: HintKey::Label("m"),
                            label: "Show",
                            priority: 90,
                        },
                        Hint {
                            key: HintKey::Label("p"),
                            label: "1Password…",
                            priority: 80,
                        },
                        Hint {
                            key: HintKey::Label("s"),
                            label: "Scope…",
                            priority: 70,
                        },
                        Hint {
                            key: HintKey::Label("d"),
                            label: "Remove…",
                            priority: 60,
                        },
                        Hint {
                            key: HintKey::Label("a"),
                            label: "Add…",
                            priority: 50,
                        },
                        Hint {
                            key: HintKey::Label("[ ]"),
                            label: "Switch tab",
                            priority: 40,
                        },
                        Hint {
                            key: HintKey::Label("Ctrl+S"),
                            label: "Save",
                            priority: 30,
                        },
                        Hint {
                            key: HintKey::Label("Esc"),
                            label: "Back",
                            priority: 20,
                        },
                    ],
                    badge: None,
                    status: None,
                    centered: true,
                },
                Tab::Accounts => HintLayer {
                    hints: vec![
                        Hint {
                            key: HintKey::Label("Space"),
                            label: "Enable / disable",
                            priority: 100,
                        },
                        Hint {
                            key: HintKey::Label("p"),
                            label: "Prefer",
                            priority: 90,
                        },
                        Hint {
                            key: HintKey::Label("/"),
                            label: "Filter",
                            priority: 80,
                        },
                        Hint {
                            key: HintKey::Label("c"),
                            label: "Manage accounts",
                            priority: 70,
                        },
                        Hint {
                            key: HintKey::Label("[ ]"),
                            label: "Switch tab",
                            priority: 60,
                        },
                        Hint {
                            key: HintKey::Label("Ctrl+S"),
                            label: "Save",
                            priority: 50,
                        },
                        Hint {
                            key: HintKey::Label("Esc"),
                            label: "Back",
                            priority: 40,
                        },
                    ],
                    badge: None,
                    status: None,
                    centered: true,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jackin_preview_domain::account::{Account, CredentialSource, DetectedKind};
    use jackin_preview_domain::agent::Provider;

    fn registry() -> AccountRegistry {
        let mut registry = AccountRegistry::default();
        let mut personal = Account::registered(
            "personal",
            "Personal",
            Provider::Anthropic,
            CredentialSource::HostEnv {
                var: "ANTHROPIC_API_KEY".into(),
                detected: DetectedKind::ClaudeApiKeyEnv,
            },
        );
        personal.default_for_provider = true;
        personal.lifecycle = jackin_preview_domain::account::Lifecycle::Available;
        let mut work = Account::registered(
            "work",
            "Work",
            Provider::Anthropic,
            CredentialSource::HostEnv {
                var: "ANTHROPIC_WORK_API_KEY".into(),
                detected: DetectedKind::ClaudeApiKeyEnv,
            },
        );
        work.lifecycle = jackin_preview_domain::account::Lifecycle::Available;
        registry.insert(personal);
        registry.insert(work);
        registry
    }

    #[test]
    fn debug_and_clone_redact_transient_environment_value() {
        let mut state = EditorState::default();
        state.env_form_open = true;
        state.env_key = "DATABASE_URL".into();
        state.env_value = "pw-fixture-only".into();

        let debug = format!("{state:?}");
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("pw-fixture-only"));

        let snapshot = state.clone();
        assert!(snapshot.env_value.is_empty());
        assert_eq!(snapshot.env_key, "DATABASE_URL");
    }

    #[test]
    fn clearing_environment_form_zeroizes_transient_input() {
        let mut state = EditorState::default();
        state.open_env_form();
        state.env_value = "transient-secret".into();
        state.env_value_input = termrock::TextInputState::sensitive();
        state.env_value_input.begin("transient-secret");

        state.clear_env_form();

        assert!(!state.env_form_open);
        assert!(state.env_value.is_empty());
        assert!(!state.env_value_input.is_editing());
    }

    #[test]
    fn tab_aliases_and_navigation_share_one_order() {
        let mut state = EditorState::default();
        for (alias, expected) in Tab::ALL.into_iter().enumerate() {
            state.select_alias(u8::try_from(alias + 1).expect("five tabs fit"));
            assert_eq!(state.tab, expected);
        }
        assert_eq!(Tab::General.previous(), Tab::Accounts);
        assert_eq!(Tab::Accounts.next(), Tab::General);
        state.tab = Tab::General;
        state.next_tab();
        assert_eq!(state.tab, Tab::Mounts);
        state.previous_tab();
        assert_eq!(state.tab, Tab::General);
        state.tab = Tab::Accounts;
        state.select_index(1);
        assert_eq!(state.tab, Tab::General);
    }

    #[test]
    fn pending_workspace_round_trips_role_and_environment_state() {
        let mut workspace = Workspace::new(7, "payments", "/workspace/payments");
        workspace.roles.default = Some("chainargos/backend".into());
        workspace.env.push(EnvVar::plain("APP_ENV", "staging"));
        workspace.role_env.insert(
            "chainargos/backend".into(),
            vec![EnvVar::plain("ROLE_FLAG", "on")],
        );

        let pending = PendingWorkspace::from_workspace(&workspace);
        assert_eq!(pending.configured_role_count(), 1);
        assert_eq!(pending.environment_count(), 2);

        let mut restored = Workspace::new(7, "other", "/other");
        pending.apply_to(&mut restored);
        assert_eq!(restored, workspace);
    }

    #[test]
    fn account_toggle_and_preference_are_policy_safe() {
        let registry = registry();
        let mut pending = PendingWorkspace::default();

        assert!(
            pending
                .effective_accounts(&registry)
                .iter()
                .any(|account| account.id == "personal")
        );
        assert_eq!(pending.toggle_account("personal", &registry), Ok(false));
        assert!(
            !pending
                .effective_accounts(&registry)
                .iter()
                .any(|account| account.id == "personal")
        );
        assert_eq!(pending.toggle_account("personal", &registry), Ok(true));

        pending.enable_account("work");
        assert_eq!(pending.prefer_account("work", &registry), Ok(()));
        assert!(
            pending
                .effective_accounts(&registry)
                .iter()
                .any(|account| account.id == "work" && account.preferred)
        );
        assert_eq!(
            pending.prefer_role_account("chainargos/backend", "work", &registry),
            Ok(())
        );
    }

    #[test]
    fn environment_staging_rejects_invalid_and_duplicate_keys() {
        let mut pending = PendingWorkspace::default();
        assert!(
            pending
                .add_environment("BAD-NAME", "secret".into())
                .is_err()
        );
        assert!(pending.env.is_empty());
        assert!(
            pending
                .add_environment("GOOD_NAME", "secret".into())
                .is_ok()
        );
        assert!(
            pending
                .add_environment("GOOD_NAME", "other".into())
                .is_err()
        );
        assert_eq!(pending.env.len(), 1);
    }
}
