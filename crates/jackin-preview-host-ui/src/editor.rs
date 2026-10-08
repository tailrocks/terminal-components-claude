//! Workspace editor state, public control ids, and EditorScreen component.

use core::{fmt, mem};
use std::collections::BTreeMap;

use ratatui::layout::Rect;
use termrock::author::{FgStep, GlyphRole, Modifier, Role, StylePatch, Ui};
use termrock::controls::{Button, Checkbox, Panel, PanelKind};
use termrock::fields::{Field, TextInput, TextInputState};
use termrock::layout::Track;
use termrock::navigation::{List, ListState, Tabs, TabsState};
use termrock::overlays::{Select, SelectState};
use termrock::{Hint, HintKey, HintLayer, Id, ItemKey};

use jackin_preview_domain::account::{AccountId, AccountRegistry};
use jackin_preview_domain::workspace::{
    AccountPolicy, EffectiveAccount, EnvValue, EnvVar, Mount, RoleName, RolePolicy, Workspace,
    env_key_error,
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

/// Durable editor state.
#[derive(PartialEq, Eq, Default)]
pub struct EditorState {
    /// Active editor tab.
    pub tab: Tab,
    /// Whether the draft has unsaved changes.
    pub dirty: bool,
    /// Whether the read-only preview is open.
    pub preview_open: bool,
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
            dirty: self.dirty,
            preview_open: self.preview_open,
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
            .field("dirty", &self.dirty)
            .field("preview_open", &self.preview_open)
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
        self.dirty = false;
        self.preview_open = false;
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

    /// Bind a new configuration explicitly, clearing any previous editor ticket.
    pub fn load_new(&mut self, pending: PendingWorkspace) {
        self.tab = Tab::General;
        self.pending = pending;
        self.original = None;
        self.reviewed = None;
        self.saving = None;
        self.dirty = true;
        self.preview_open = false;
        self.clear_env_form();
    }

    /// Whether this draft creates a configuration rather than replacing a loaded one.
    pub const fn is_create(&self) -> bool {
        self.original.is_none()
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
}

impl Default for PendingWorkspace {
    fn default() -> Self {
        Self {
            name: "payments-platform".into(),
            workdir: "/workspace/payments-platform".into(),
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
                    row.label(&format!("{name} •"));
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

        // 2. Draw tab content
        match editor.tab {
            Tab::General => Self::draw_general(ui, area, editor, focused),
            Tab::Mounts => Self::draw_mounts(ui, area, editor, focused),
            Tab::Roles => Self::draw_roles(ui, area, editor, world, focused),
            Tab::Environments => Self::draw_environments(ui, area, editor, world, focused),
            Tab::Accounts => Self::draw_accounts(ui, area, editor, world, focused),
        }

        // 3. Draw bottom action buttons (Cancel and Save…)
        let cancel_area = Rect::new(97, 37, 8, 1);
        let save_area = Rect::new(108, 37, 8, 1);
        Button::new(FORM.sub("cancel"), "Cancel").draw(ui, cancel_area);
        Button::new(SAVE, "Save…").draw(ui, save_area);
    }

    fn draw_general(ui: &mut Ui<'_>, _area: Rect, editor: &EditorState, _focused: bool) {
        // Name field
        let mut name_state = TextInputState::default();
        name_state.begin(&editor.pending.name);
        Field::new("Name", TextInput::new(FORM.sub("name")))
            .required(true)
            .help("Directory basename by default")
            .draw(ui, Rect::new(4, 6, 70, 3), &name_state);

        // Working directory
        let workdir_state = ListState::default();
        let workdir_items = [
            ("Working directory *", false),
            (editor.pending.workdir.as_str(), true),
            ("Inside the Construct", false),
        ];
        List::new(FORM.sub("workdir-list"))
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
            .draw(ui, Rect::new(4, 10, 60, 3), &workdir_state, &workdir_items);

        // Checkboxes
        Checkbox::new(KEEP_AWAKE, "Keep awake")
            .checked(editor.pending.keep_awake)
            .draw(ui, Rect::new(4, 14, 25, 1));

        let badge_state = ListState::default();
        List::new(FORM.sub("macos-badge"))
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::BorderStrong);
                row.label_patched("macOS only", &p);
            })
            .draw(ui, Rect::new(34, 14, 15, 1), &badge_state, &[()]);

        Checkbox::new(GIT_PULL, "Git pull before launch")
            .checked(editor.pending.git_pull)
            .draw(ui, Rect::new(4, 15, 30, 1));

        // On dirty exit
        let dirty_exit_label_state = ListState::default();
        List::new(FORM.sub("dirty-exit-label"))
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                row.label_patched("On dirty exit", &p);
            })
            .draw(ui, Rect::new(6, 17, 20, 1), &dirty_exit_label_state, &[()]);

        let select_state = SelectState::default();
        Select::new(FORM.sub("on-dirty-exit")).draw(
            ui,
            Rect::new(4, 18, 48, 1),
            &select_state,
            &["ask · show the exit dialog"],
        );

        // Drawn last so the focus ring visits the workdir row before the
        // picker button (TABS → NAME → WORKDIR → KEEP_AWAKE); the rect is
        // unchanged.
        Button::new(FORM.sub("choose"), "Choose…").draw(ui, Rect::new(65, 11, 10, 1));
    }

    fn draw_mounts(ui: &mut Ui<'_>, _area: Rect, editor: &EditorState, _focused: bool) {
        #[derive(Clone)]
        enum MountRow {
            Header,
            Mount {
                dest: &'static str,
                mode: &'static str,
                isolation: &'static str,
                kind: &'static str,
                source: &'static str,
                selected: bool,
                dirty: bool,
            },
            Add,
        }

        let is_dirty = editor.dirty;
        let rows = vec![
            MountRow::Header,
            MountRow::Mount {
                dest: "/workspace/payments-platform",
                mode: if is_dirty { "ro" } else { "rw" },
                isolation: if is_dirty { "clone" } else { "worktree" },
                kind: "host",
                source: "~/src/payments-platform",
                selected: true,
                dirty: is_dirty,
            },
            MountRow::Mount {
                dest: "/workspace/libs",
                mode: "ro",
                isolation: "shared",
                kind: "host",
                source: "~/src/shared-libs",
                selected: false,
                dirty: false,
            },
            MountRow::Add,
        ];

        let state = ListState::default();
        List::new(FORM.sub("mounts-list"))
            .row(|item, row| match item {
                MountRow::Header => {
                    let mut cols = row.columns_with_gap(
                        &[
                            Track::Fixed(36),
                            Track::Fixed(4),
                            Track::Fixed(9),
                            Track::Fixed(6),
                            Track::Flex(1),
                        ],
                        2,
                    );
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                    cols.cell(0).patch(&p).text("    Destination");
                    cols.cell(1).patch(&p).text("Mode");
                    cols.cell(2).patch(&p).text("Isolation");
                    cols.cell(3).patch(&p).text("Kind");
                    cols.cell(4).patch(&p).text("Source");
                }
                MountRow::Mount {
                    dest,
                    mode,
                    isolation,
                    kind,
                    source,
                    selected,
                    dirty,
                } => {
                    if *selected {
                        row.marker(GlyphRole::Chosen);
                        if *dirty {
                            let mut cols = row.columns_with_gap(
                                &[
                                    Track::Fixed(2),
                                    Track::Fixed(34),
                                    Track::Fixed(4),
                                    Track::Fixed(9),
                                    Track::Fixed(6),
                                    Track::Flex(1),
                                ],
                                2,
                            );
                            let p_bullet = StylePatch::new().set_fg(Role::Warning);
                            cols.cell(0).patch(&p_bullet).text("•");
                            cols.cell(1).text(dest);
                            let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                            cols.cell(2).patch(&p_sec).text(mode);
                            cols.cell(3).patch(&p_sec).text(isolation);
                            let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                            cols.cell(4).patch(&p_muted).text(kind);
                            cols.cell(5).patch(&p_muted).text(source);
                        } else {
                            let mut cols = row.columns_with_gap(
                                &[
                                    Track::Fixed(36),
                                    Track::Fixed(4),
                                    Track::Fixed(9),
                                    Track::Fixed(6),
                                    Track::Flex(1),
                                ],
                                2,
                            );
                            cols.cell(0).text(&format!("  {dest}"));
                            let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                            cols.cell(1).patch(&p_sec).text(mode);
                            cols.cell(2).patch(&p_sec).text(isolation);
                            let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                            cols.cell(3).patch(&p_muted).text(kind);
                            cols.cell(4).patch(&p_muted).text(source);
                        }
                    } else {
                        let mut cols = row.columns_with_gap(
                            &[
                                Track::Fixed(36),
                                Track::Fixed(4),
                                Track::Fixed(9),
                                Track::Fixed(6),
                                Track::Flex(1),
                            ],
                            2,
                        );
                        cols.cell(0).text(&format!("    {dest}"));
                        let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                        cols.cell(1).patch(&p_sec).text(mode);
                        cols.cell(2).patch(&p_sec).text(isolation);
                        let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                        cols.cell(3).patch(&p_muted).text(kind);
                        cols.cell(4).patch(&p_muted).text(source);
                    }
                }
                MountRow::Add => {
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    row.label_patched("    + Add mount", &p);
                }
            })
            .draw(ui, Rect::new(4, 6, 114, 5), &state, &rows);
    }

    fn draw_roles(
        ui: &mut Ui<'_>,
        _area: Rect,
        _editor: &EditorState,
        _world: &World,
        _focused: bool,
    ) {
        // Roles header
        let header_state = ListState::default();
        List::new(FORM.sub("roles-header"))
            .row(|_, row| {
                let p_bold = StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Secondary))
                    .add(Modifier::BOLD);
                row.label_patched("Allowed roles  3 of 46", &p_bold);
                row.meta("default ★ the-architect");
            })
            .draw(ui, Rect::new(4, 6, 112, 1), &header_state, &[()]);

        // Roles list items
        #[derive(Clone)]
        struct RoleItem {
            name: &'static str,
            desc: &'static str,
            allowed: bool,
            is_default: bool,
            selected: bool,
        }

        let mut items = vec![
            RoleItem {
                name: "the-architect",
                desc: "registry · trusted · Full-stack design and refactoring; Claude Code default",
                allowed: true,
                is_default: true,
                selected: true,
            },
            RoleItem {
                name: "backend",
                desc: "registry · trusted · Rust and Postgres services",
                allowed: true,
                is_default: false,
                selected: false,
            },
            RoleItem {
                name: "reviewer",
                desc: "registry · trusted · Read-mostly code review with limited write scope",
                allowed: true,
                is_default: false,
                selected: false,
            },
            RoleItem {
                name: "sre",
                desc: "registry · trusted · Infrastructure, Terraform, Kubernetes",
                allowed: false,
                is_default: false,
                selected: false,
            },
            RoleItem {
                name: "data-eng",
                desc: "! load error · trust required",
                allowed: false,
                is_default: false,
                selected: false,
            },
            RoleItem {
                name: "writer",
                desc: "~/roles/writer · not in registry",
                allowed: false,
                is_default: false,
                selected: false,
            },
        ];

        for i in 1..=40 {
            let desc: &'static str = match (i - 1) % 5 {
                0 => Box::leak(
                    format!("registry · trusted · ledger service agent #{i}").into_boxed_str(),
                ),
                1 => Box::leak(
                    format!("registry · trusted · search service agent #{i}").into_boxed_str(),
                ),
                2 => Box::leak(
                    format!("registry · trusted · notify service agent #{i}").into_boxed_str(),
                ),
                3 => Box::leak(
                    format!("registry · trusted · ingest service agent #{i}").into_boxed_str(),
                ),
                _ => Box::leak(
                    format!("registry · trusted · auth service agent #{i}").into_boxed_str(),
                ),
            };
            items.push(RoleItem {
                name: Box::leak(format!("svc-{i:03}").into_boxed_str()),
                desc,
                allowed: false,
                is_default: false,
                selected: false,
            });
        }

        let state = ListState::default();
        List::new(FORM.sub("roles-list"))
            .row(|item: &RoleItem, row| {
                if item.selected {
                    row.marker(GlyphRole::Chosen);
                    let mut cols = row.columns_with_gap(&[Track::Fixed(22), Track::Flex(1)], 2);
                    let mark = if item.allowed { "[✓]" } else { "[ ]" };
                    let star = if item.is_default { " ★" } else { "" };
                    cols.cell(0).text(&format!(" {mark} {}{star}", item.name));
                    let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                    cols.cell(1).patch(&p_muted).text(item.desc);
                } else {
                    let mut cols = row.columns_with_gap(&[Track::Fixed(24), Track::Flex(1)], 2);
                    let mark = if item.allowed { "[✓]" } else { "[ ]" };
                    let star = if item.is_default { " ★" } else { "" };
                    cols.cell(0).text(&format!("   {mark} {}{star}", item.name));
                    let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                    cols.cell(1).patch(&p_muted).text(item.desc);
                }
            })
            .draw(ui, Rect::new(2, 7, 116, 30), &state, &items);
    }

    fn draw_environments(
        ui: &mut Ui<'_>,
        _area: Rect,
        _editor: &EditorState,
        _world: &World,
        _focused: bool,
    ) {
        #[derive(Clone)]
        enum EnvItem {
            WorkspaceHeader,
            Var {
                key: &'static str,
                val: &'static str,
                src: &'static str,
                selected: bool,
            },
            AddVar,
            RoleHeader,
            RoleGroup(&'static str),
            RoleVar {
                key: &'static str,
                val: &'static str,
                src: &'static str,
            },
            AddRoleVar,
            AddRoleOverride,
        }

        let rows = vec![
            EnvItem::WorkspaceHeader,
            EnvItem::Var {
                key: "DATABASE_URL",
                val: "****************",
                src: "plain",
                selected: true,
            },
            EnvItem::Var {
                key: "STRIPE_KEY",
                val: "****************",
                src: "[op] Engineering › Stripe · sandbox › credential",
                selected: false,
            },
            EnvItem::Var {
                key: "LOG_LEVEL",
                val: "*****",
                src: "plain",
                selected: false,
            },
            EnvItem::Var {
                key: "GH_TOKEN",
                val: "$GH_TOKEN",
                src: "host env",
                selected: false,
            },
            EnvItem::AddVar,
            EnvItem::RoleHeader,
            EnvItem::RoleGroup("backend"),
            EnvItem::RoleVar {
                key: "OPENAI_API_KEY",
                val: "****************",
                src: "[op] Engineering › OpenAI · Codex Primary › credential",
            },
            EnvItem::AddRoleVar,
            EnvItem::AddRoleOverride,
        ];

        let state = ListState::default();
        List::new(FORM.sub("env-list"))
            .row(|item, row| match item {
                EnvItem::WorkspaceHeader => {
                    let p_bold = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Secondary))
                        .add(Modifier::BOLD);
                    row.label_patched("    Workspace", &p_bold);
                    row.meta("4 vars");
                }
                EnvItem::Var {
                    key,
                    val,
                    src,
                    selected,
                } => {
                    if *selected {
                        row.marker(GlyphRole::Chosen);
                        let mut cols = row.columns_with_gap(
                            &[Track::Fixed(22), Track::Fixed(24), Track::Flex(1)],
                            2,
                        );
                        cols.cell(0).text(&format!("  {key}"));
                        let p_val = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                        cols.cell(1).patch(&p_val).text(val);
                        let p_src = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                        cols.cell(2).patch(&p_src).text(src);
                    } else {
                        let mut cols = row.columns_with_gap(
                            &[Track::Fixed(24), Track::Fixed(24), Track::Flex(1)],
                            2,
                        );
                        cols.cell(0).text(&format!("    {key}"));
                        let p_val = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                        cols.cell(1).patch(&p_val).text(val);
                        let p_src = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                        cols.cell(2).patch(&p_src).text(src);
                    }
                }
                EnvItem::AddVar => {
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    row.label_patched("    + Add environment variable", &p);
                }
                EnvItem::RoleHeader => {
                    let p_bold = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Secondary))
                        .add(Modifier::BOLD);
                    row.label_patched("    Role overrides", &p_bold);
                    row.meta("1 configured · 46 in the registry");
                }
                EnvItem::RoleGroup(role) => {
                    let p_bold = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Secondary))
                        .add(Modifier::BOLD);
                    row.label_patched(&format!("  ▾ Role: {role}"), &p_bold);
                    row.meta("1 var");
                }
                EnvItem::RoleVar { key, val, src } => {
                    let mut cols = row
                        .columns_with_gap(&[Track::Fixed(24), Track::Fixed(24), Track::Flex(1)], 2);
                    cols.cell(0).text(&format!("    {key}"));
                    let p_val = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    cols.cell(1).patch(&p_val).text(val);
                    let p_src = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                    cols.cell(2).patch(&p_src).text(src);
                }
                EnvItem::AddRoleVar => {
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    row.label_patched("    + Add backend environment variable", &p);
                }
                EnvItem::AddRoleOverride => {
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    row.label_patched("    + Add role override…", &p);
                }
            })
            .draw(ui, Rect::new(2, 6, 116, 13), &state, &rows);
    }

    fn draw_accounts(
        ui: &mut Ui<'_>,
        _area: Rect,
        _editor: &EditorState,
        _world: &World,
        _focused: bool,
    ) {
        #[derive(Clone)]
        enum AccRow {
            Header,
            Provider {
                name: &'static str,
                meta: &'static str,
            },
            Account {
                name: &'static str,
                is_default: bool,
                policy: &'static str,
                status: &'static str,
                checked: bool,
                selected: bool,
            },
        }

        let rows = vec![
            AccRow::Header,
            AccRow::Provider {
                name: "Anthropic / Claude",
                meta: "2 active · picker at session start",
            },
            AccRow::Account {
                name: "Personal",
                is_default: true,
                policy: "inherited default",
                status: "ready",
                checked: true,
                selected: true,
            },
            AccRow::Account {
                name: "Archived contractor laptop …",
                is_default: false,
                policy: "available",
                status: "disabled globally",
                checked: false,
                selected: false,
            },
            AccRow::Account {
                name: "Work",
                is_default: false,
                policy: "enabled here",
                status: "ready",
                checked: true,
                selected: false,
            },
            AccRow::Provider {
                name: "OpenAI",
                meta: "1 active",
            },
            AccRow::Account {
                name: "Primary",
                is_default: true,
                policy: "inherited default",
                status: "ready",
                checked: true,
                selected: false,
            },
            AccRow::Account {
                name: "Experiments",
                is_default: false,
                policy: "available",
                status: "ready",
                checked: false,
                selected: false,
            },
            AccRow::Provider {
                name: "Amp",
                meta: "none active",
            },
            AccRow::Account {
                name: "host login",
                is_default: false,
                policy: "discovered on host",
                status: "ready",
                checked: false,
                selected: false,
            },
            AccRow::Provider {
                name: "xAI / Grok",
                meta: "1 active",
            },
            AccRow::Account {
                name: "Team",
                is_default: true,
                policy: "inherited default",
                status: "ready",
                checked: true,
                selected: false,
            },
            AccRow::Provider {
                name: "OpenCode",
                meta: "1 active",
            },
            AccRow::Account {
                name: "Go subscription",
                is_default: true,
                policy: "inherited default",
                status: "ready",
                checked: true,
                selected: false,
            },
            AccRow::Account {
                name: "ci-bot",
                is_default: false,
                policy: "discovered on host",
                status: "unsupported",
                checked: false,
                selected: false,
            },
            AccRow::Provider {
                name: "Moonshot / Kimi",
                meta: "none active",
            },
            AccRow::Account {
                name: "host login",
                is_default: false,
                policy: "discovered on host",
                status: "needs secret",
                checked: false,
                selected: false,
            },
            AccRow::Provider {
                name: "Z.AI",
                meta: "none active",
            },
            AccRow::Account {
                name: "host login",
                is_default: false,
                policy: "discovered on host",
                status: "ready",
                checked: false,
                selected: false,
            },
            AccRow::Provider {
                name: "MiniMax",
                meta: "none active",
            },
            AccRow::Account {
                name: "host login",
                is_default: false,
                policy: "discovered on host",
                status: "unavailable",
                checked: false,
                selected: false,
            },
        ];

        let state = ListState::default();
        List::new(FORM.sub("accounts-list"))
            .row(|item, row| match item {
                AccRow::Header => {
                    let p_bold = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Secondary))
                        .add(Modifier::BOLD);
                    row.label_patched(
                        "Active accounts  5 effective · 4 inherited · 1 enabled here",
                        &p_bold,
                    );
                    row.meta("registry in Accounts (c)");
                }
                AccRow::Provider { name, meta } => {
                    let p_bold = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Secondary))
                        .add(Modifier::BOLD);
                    row.label_patched(name, &p_bold);
                    row.meta(meta);
                }
                AccRow::Account {
                    name,
                    is_default,
                    policy,
                    status,
                    checked,
                    selected,
                } => {
                    let mark = if *checked { "[✓]" } else { "[ ]" };
                    let star = if *is_default { "★" } else { " " };
                    if *selected {
                        row.marker(GlyphRole::Chosen);
                        let mut cols = row.columns_with_gap(
                            &[Track::Fixed(37), Track::Fixed(22), Track::Flex(1)],
                            2,
                        );
                        cols.cell(0).text(&format!(" {mark} {name:<29} {star}"));
                        let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                        cols.cell(1).patch(&p_muted).text(policy);
                        cols.cell(2).patch(&p_muted).text(status);
                    } else {
                        let mut cols = row.columns_with_gap(
                            &[Track::Fixed(39), Track::Fixed(22), Track::Flex(1)],
                            2,
                        );
                        cols.cell(0).text(&format!("   {mark} {name:<29} {star}"));
                        let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                        cols.cell(1).patch(&p_muted).text(policy);
                        cols.cell(2).patch(&p_muted).text(status);
                    }
                }
            })
            .draw(ui, Rect::new(4, 6, 114, 25), &state, &rows);
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

                Button::new(CFG_FORM.sub("cancel"), "Cancel").draw(
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

    /// Editor hints for the bottom hint bar.
    pub fn hints(editor: &EditorState) -> HintLayer {
        if editor.preview_open {
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
        } else {
            match editor.tab {
                Tab::General => HintLayer {
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
