use core::{fmt, mem};
use std::collections::BTreeMap;

use termrock::Id;

use jackin_preview_domain::account::{AccountId, AccountRegistry};
use jackin_preview_domain::workspace::{
    AccountPolicy, EffectiveAccount, EnvValue, EnvVar, Mount, RoleName, RolePolicy, Workspace,
    env_key_error,
};

pub const ROOT: Id = Id::root("jackin.editor");
pub const FORM: Id = ROOT.sub("form");
pub const CFG: Id = Id::root("editor.cfg");
pub const CFG_FORM: Id = CFG.sub("form");
pub const SAVE: Id = CFG_FORM.sub("save");
pub const TABS: Id = ROOT.sub("tabs");
pub const TAB_GENERAL: Id = TABS.sub("general");
pub const TAB_MOUNTS: Id = TABS.sub("mounts");
pub const TAB_ROLES: Id = TABS.sub("roles");
pub const TAB_ENVIRONMENTS: Id = TABS.sub("environments");
pub const TAB_ACCOUNTS: Id = TABS.sub("accounts");
pub const ENV_KEY: Id = FORM.sub("env-key");
pub const ENV_SOURCE: Id = FORM.sub("env-source");
pub const ENV_VALUE: Id = FORM.sub("env-value");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    General,
    Mounts,
    Roles,
    Environments,
    Accounts,
}

impl Tab {
    pub const ALL: [Self; 5] = [
        Self::General,
        Self::Mounts,
        Self::Roles,
        Self::Environments,
        Self::Accounts,
    ];

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

    pub const fn alias(self) -> u8 {
        match self {
            Self::General => 1,
            Self::Mounts => 2,
            Self::Roles => 3,
            Self::Environments => 4,
            Self::Accounts => 5,
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::General => Self::Mounts,
            Self::Mounts => Self::Roles,
            Self::Roles => Self::Environments,
            Self::Environments => Self::Accounts,
            Self::Accounts => Self::General,
        }
    }

    pub const fn previous(self) -> Self {
        match self {
            Self::General => Self::Accounts,
            Self::Mounts => Self::General,
            Self::Roles => Self::Mounts,
            Self::Environments => Self::Roles,
            Self::Accounts => Self::Environments,
        }
    }

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

#[derive(PartialEq, Eq, Default)]
pub struct EditorState {
    pub tab: Tab,
    pub dirty: bool,
    pub preview_open: bool,
    pub env_form_open: bool,
    pub env_key: String,
    pub env_value: String,
    pub env_key_input: termrock::TextInputState,
    pub env_value_input: termrock::TextInputState,
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

    pub const fn select_alias(&mut self, index: u8) {
        if let Some(tab) = Tab::from_alias(index) {
            self.tab = tab;
        }
    }

    pub const fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }

    pub const fn previous_tab(&mut self) {
        self.tab = self.tab.previous();
    }

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

    pub fn mark_dirty(&mut self) {
        self.reviewed = None;
        self.dirty = true;
        self.preview_open = false;
    }

    pub fn mark_saved(&mut self) {
        self.reviewed = None;
        self.dirty = false;
        self.preview_open = false;
    }

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

    pub fn close_preview(&mut self) {
        self.preview_open = false;
        self.reviewed = None;
    }

    pub fn discard_draft(&mut self) {
        if let Some(original) = &self.original {
            self.pending = PendingWorkspace::from_workspace(original);
        }
        self.dirty = false;
        self.preview_open = false;
        self.reviewed = None;
        self.clear_env_form();
    }

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

    pub const fn is_create(&self) -> bool {
        self.original.is_none()
    }

    pub const fn is_saving(&self) -> bool {
        self.saving.is_some()
    }

    pub fn workspace_id(&self) -> Option<jackin_preview_domain::workspace::WorkspaceId> {
        self.original.as_ref().map(|workspace| workspace.id)
    }

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

    pub fn open_env_form(&mut self) {
        self.clear_env_form();
        self.env_form_open = true;
        self.env_value_input = termrock::TextInputState::sensitive();
    }

    pub fn clear_env_form(&mut self) {
        self.env_form_open = false;
        self.env_key.clear();
        self.env_key_input = termrock::TextInputState::default();
        self.env_value_input.zeroize();
        self.env_value_input = termrock::TextInputState::default();
        wipe_string(&mut self.env_value);
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingWorkspace {
    pub name: String,
    pub workdir: String,
    pub mounts: Vec<Mount>,
    pub roles: RolePolicy,
    pub env: Vec<EnvVar>,
    pub role_env: BTreeMap<RoleName, Vec<EnvVar>>,
    pub accounts: AccountPolicy,
    pub keep_awake: bool,
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
            keep_awake: false,
        }
    }
}

impl PendingWorkspace {
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
        }
    }

    pub fn apply_to(&self, workspace: &mut Workspace) {
        workspace.name = self.name.clone();
        workspace.workdir = self.workdir.clone();
        workspace.mounts = self.mounts.clone();
        workspace.roles = self.roles.clone();
        workspace.env = self.env.clone();
        workspace.role_env = self.role_env.clone();
        workspace.accounts = self.accounts.clone();
        workspace.keep_awake = self.keep_awake;
    }

    pub fn into_workspace(self, id: u32) -> Workspace {
        let mut workspace = Workspace::new(id, &self.name, &self.workdir);
        workspace.mounts = self.mounts;
        workspace.roles = self.roles;
        workspace.env = self.env;
        workspace.role_env = self.role_env;
        workspace.accounts = self.accounts;
        workspace.keep_awake = self.keep_awake;
        workspace
    }

    pub fn effective_accounts(&self, registry: &AccountRegistry) -> Vec<EffectiveAccount> {
        let mut workspace = Workspace::new(0, &self.name, &self.workdir);
        workspace.mounts = self.mounts.clone();
        workspace.roles = self.roles.clone();
        workspace.env = self.env.clone();
        workspace.role_env = self.role_env.clone();
        workspace.accounts = self.accounts.clone();
        workspace.effective_accounts(registry)
    }

    pub fn enable_account(&mut self, id: impl Into<AccountId>) {
        let id = id.into();
        self.accounts.disabled_defaults.remove(&id);
        self.accounts.enabled.insert(id);
    }

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

    pub fn add_environment(&mut self, key: &str, value: String) -> Result<(), String> {
        add_env(&mut self.env, key, value)
    }

    pub fn add_role_environment(
        &mut self,
        role: impl Into<RoleName>,
        key: &str,
        value: String,
    ) -> Result<(), String> {
        let role = role.into();
        add_env(self.role_env.entry(role).or_default(), key, value)
    }

    pub fn configured_role_count(&self) -> usize {
        self.role_env.values().filter(|env| !env.is_empty()).count()
    }

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
