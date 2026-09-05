//! Workspace editor state and public control ids.

use core::{fmt, mem};

use junie_tui::Id;

use crate::domain::account::{AccountId, AccountRegistry};
use crate::domain::workspace::{AccountPolicy, EffectiveAccount, EnvVar, Mount, Workspace};

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

    /// Open a fresh environment-variable form, dropping any previous input.
    pub(crate) fn open_env_form(&mut self) {
        self.clear_env_form();
        self.env_form_open = true;
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
        self.env_value_input.zeroize();
        wipe_string(&mut self.env_value);
    }

    /// Move the committed transient value into its durable draft owner.
    pub(crate) fn take_env_value(&mut self) -> String {
        self.env_value_input.zeroize();
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
    /// Workspace environment draft. Persisted only after the editor save job succeeds.
    pub env: Vec<EnvVar>,
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
            env: vec![],
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
