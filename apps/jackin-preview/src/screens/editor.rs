//! Workspace editor state and public control ids.

use core::{fmt, mem};
use std::collections::BTreeMap;

use junie_tui::Id;

use crate::domain::account::{AccountId, AccountRegistry};
use crate::domain::workspace::{
    AccountPolicy, EffectiveAccount, EnvValue, EnvVar, Mount, RoleName, RolePolicy, Workspace,
    env_key_error,
};

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
    /// Canonical editor tab order.  Keep this in one place so keyboard and
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
    pub env_key_input: junie_tui::TextInputState,
    /// Sensitive runtime state for the value input.
    pub env_value_input: junie_tui::TextInputState,
    /// Mutable workspace draft projected by the editor controls.
    pub pending: PendingWorkspace,
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
            env_value_input: junie_tui::TextInputState::sensitive(),
            pending: self.pending.clone(),
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
    ///
    /// The old shell passed the *destination offset* here (`1` means Mounts,
    /// `2` means Roles, …).  Keep that contract for compatibility; new code
    /// should use [`Self::select_alias`] or [`Tab::next`] instead.
    pub const fn select_index(&mut self, index: u8) {
        self.tab = match (self.tab, index) {
            // The shell passes `1` when wrapping from Accounts.  The old
            // value-only match sent that transition back to Mounts.
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
        self.clear_env_form();
    }

    /// Mark the current draft as changed and close any stale preview.
    pub const fn mark_dirty(&mut self) {
        self.dirty = true;
        self.preview_open = false;
    }

    /// Mark a successful save and close the preview.
    pub const fn mark_saved(&mut self) {
        self.dirty = false;
        self.preview_open = false;
    }

    /// Open the save preview only when there are pending changes.
    pub const fn open_preview(&mut self) -> bool {
        if self.dirty {
            self.preview_open = true;
        }
        self.preview_open
    }

    /// Close a save preview without discarding the draft.
    pub const fn close_preview(&mut self) {
        self.preview_open = false;
    }

    /// Open a fresh environment-variable form, dropping any previous input.
    pub(crate) fn open_env_form(&mut self) {
        self.clear_env_form();
        self.env_form_open = true;
        self.env_value_input = junie_tui::TextInputState::sensitive();
    }

    /// Cancel and clear all transient environment-variable input.
    pub(crate) fn clear_env_form(&mut self) {
        self.env_form_open = false;
        self.env_key.clear();
        self.env_key_input = junie_tui::TextInputState::default();
        self.env_value_input.zeroize();
        self.env_value_input = junie_tui::TextInputState::default();
        wipe_string(&mut self.env_value);
    }

    /// Drop the transient value while retaining the rest of the form.
    pub(crate) fn discard_env_value(&mut self) {
        self.env_value_input.cancel();
        self.env_value_input.zeroize();
        wipe_string(&mut self.env_value);
    }

    /// Move the committed transient value into its durable draft owner.
    pub(crate) fn take_env_value(&mut self) -> String {
        self.env_value_input.zeroize();
        self.env_value_input = junie_tui::TextInputState::sensitive();
        mem::take(&mut self.env_value)
    }
}

fn wipe_string(value: &mut String) {
    let mut secret = junie_tui::Secret::new(mem::take(value));
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
}

impl Default for PendingWorkspace {
    fn default() -> Self {
        Self {
            name: "payments-platform".into(),
            workdir: "/Users/alexey/src/payments-platform".into(),
            mounts: vec![Mount::host(
                "/Users/alexey/src/payments-platform",
                "/Users/alexey/src/payments-platform",
            )],
            roles: RolePolicy::default(),
            env: vec![],
            role_env: BTreeMap::new(),
            accounts: AccountPolicy::default(),
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
    }

    /// Consume this draft into a persisted workspace with `id`.
    pub fn into_workspace(self, id: u32) -> Workspace {
        let mut workspace = Workspace::new(id, &self.name, &self.workdir);
        workspace.mounts = self.mounts;
        workspace.roles = self.roles;
        workspace.env = self.env;
        workspace.role_env = self.role_env;
        workspace.accounts = self.accounts;
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
                // A provider default remains inherited when it is switched
                // back on; putting it in `enabled` would change its origin.
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
    let mut secret = junie_tui::Secret::new(value);
    secret.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::{Account, CredentialSource, DetectedKind};
    use crate::domain::agent::Provider;

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
        personal.lifecycle = crate::domain::account::Lifecycle::Available;
        let mut work = Account::registered(
            "work",
            "Work",
            Provider::Anthropic,
            CredentialSource::HostEnv {
                var: "ANTHROPIC_WORK_API_KEY".into(),
                detected: DetectedKind::ClaudeApiKeyEnv,
            },
        );
        work.lifecycle = crate::domain::account::Lifecycle::Available;
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
        state.env_value_input = junie_tui::TextInputState::sensitive();
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
