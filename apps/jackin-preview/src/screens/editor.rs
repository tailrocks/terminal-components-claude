//! Workspace editor state and public control ids.

use junie_tui::Id;

use crate::domain::account::{AccountId, AccountRegistry};
use crate::domain::workspace::{AccountPolicy, EffectiveAccount, Mount, Workspace};

/// Editor root.
pub const ROOT: Id = Id::root("jackin.editor");
/// Editor form root retained as a stable namespace for nested controls.
pub const FORM: Id = ROOT.sub("form");
/// Save action.
pub const SAVE: Id = Id::root("editor.cfg").sub("form").sub("save");
/// Editor tabs.
pub const TABS: Id = ROOT.sub("tabs");
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

/// Durable editor state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorState {
    pub tab: Tab,
    pub dirty: bool,
    pub preview_open: bool,
    pub env_visible: bool,
    pub env_form_open: bool,
    pub env_key: String,
    pub env_value: String,
    pub env_key_input: junie_tui::TextInputState,
    pub env_value_input: junie_tui::TextInputState,
    /// Mutable workspace draft projected by the editor controls.
    pub pending: PendingWorkspace,
}

impl EditorState {
    /// Select a tab by one-based fixture index.
    pub const fn select_index(&mut self, index: u8) {
        self.tab = match index {
            1 => Tab::Mounts,
            2 => Tab::Roles,
            3 => Tab::Environments,
            4 | 5 => Tab::Accounts,
            _ => Tab::General,
        };
    }
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
            accounts: AccountPolicy::default(),
        }
    }
}

impl PendingWorkspace {
    /// Effective accounts projected with current registry metadata.
    pub fn effective_accounts(&self, registry: &AccountRegistry) -> Vec<EffectiveAccount> {
        let mut workspace = Workspace::new(0, &self.name, &self.workdir);
        workspace.mounts = self.mounts.clone();
        workspace.accounts = self.accounts.clone();
        workspace.effective_accounts(registry)
    }

    /// Set a proposed account as enabled in this workspace.
    pub fn enable_account(&mut self, id: impl Into<AccountId>) {
        self.accounts.enabled.insert(id.into());
    }
}
