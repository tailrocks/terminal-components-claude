//! Accounts route state and controls.

use core::fmt;

use junie_tui::{Id, ListState, TextInputState};

use crate::domain::onepassword::OpReference;

/// Accounts list control.
pub const LIST: Id = Id::root("jackin.accounts.list");
/// Account form root.
pub const FORM: Id = Id::root("jackin.accounts.form");
/// Account form save action.
pub const SAVE: Id = FORM.sub("save");
/// Provider selection control.
pub const PROVIDER: Id = FORM.sub("provider");
/// Credential-source selection control.
pub const SOURCE: Id = FORM.sub("source");
/// 1Password source control.
pub const OP: Id = FORM.sub("op");
/// New-account opener.
pub const START: Id = FORM.sub("start");
/// Display-name field.
pub const NAME: Id = FORM.sub("name");
/// Agent/runtime selection control.
pub const AGENT: Id = FORM.sub("agent");
/// Local-folder field.
pub const FOLDER: Id = FORM.sub("folder");
/// Plain API-key field.
pub const SECRET: Id = FORM.sub("secret");

/// Durable accounts route state.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct AccountsState {
    /// Public-list selection state owned by this route.
    pub list: ListState,
    pub form_open: bool,
    /// Whether the opener has handed focus to the editable form.
    pub started: bool,
    pub editing: Option<String>,
    pub draft_name: String,
    pub masked_input: String,
    pub pending_refresh: Option<String>,
    pub remove_confirmation: Option<String>,
    /// Controlled public text field state for the display name.
    pub name_input: TextInputState,
    /// Controlled public text field state for a local folder.
    pub folder_input: TextInputState,
    /// Controlled public text field state for a transient API key.
    pub secret_input: TextInputState,
    /// Provider selector list state.
    pub provider_list: ListState,
    /// Credential-source selector list state.
    pub source_list: ListState,
    /// Selected registerable provider (Anthropic, OpenAI, xAI, OpenCode).
    pub provider_index: u8,
    /// Credential source choice (1Password, folder, API key).
    pub source_index: u8,
    /// Selected 1Password item metadata.
    pub selected_op: Option<OpReference>,
    /// Selected item label in the 1Password browser.
    pub op_item: String,
    /// Current 1Password browser level.
    pub op_stage: u8,
    /// Stable selected account id for semantic commands.
    pub selected_id: Option<String>,
}

impl fmt::Debug for AccountsState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountsState")
            .field("list", &self.list)
            .field("form_open", &self.form_open)
            .field("started", &self.started)
            .field("editing", &self.editing)
            .field("draft_name", &self.draft_name)
            .field("masked_input", &"[redacted]")
            .field("pending_refresh", &self.pending_refresh)
            .field("remove_confirmation", &self.remove_confirmation)
            .field("name_input", &self.name_input)
            .field("folder_input", &self.folder_input)
            .field("secret_input", &self.secret_input)
            .field("provider_list", &self.provider_list)
            .field("source_list", &self.source_list)
            .field("provider_index", &self.provider_index)
            .field("source_index", &self.source_index)
            .field("selected_op", &self.selected_op)
            .field("op_item", &self.op_item)
            .field("op_stage", &self.op_stage)
            .field("selected_id", &self.selected_id)
            .finish()
    }
}

impl AccountsState {
    /// Open a new account form.
    pub fn open_new(&mut self) {
        self.form_open = true;
        self.started = false;
        self.editing = None;
        self.draft_name.clear();
        self.masked_input.clear();
        self.name_input = TextInputState::default();
        self.folder_input = TextInputState::default();
        self.secret_input = TextInputState::default();
        self.provider_list = ListState::default();
        self.source_list = ListState::default();
        self.provider_index = 0;
        self.source_index = 0;
        self.selected_op = None;
        self.op_item.clear();
        self.op_stage = 0;
        self.selected_id = None;
        self.pending_refresh = None;
        self.remove_confirmation = None;
    }

    /// Close the form without writing a credential.
    pub const fn close(&mut self) {
        self.form_open = false;
    }
}
