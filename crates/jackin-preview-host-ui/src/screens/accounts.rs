use core::fmt;

use termrock::{Id, ListState, TextInputState};

use jackin_preview_domain::onepassword::OpReference;

pub const LIST: Id = Id::root("jackin.accounts.list");
pub const FORM: Id = Id::root("jackin.accounts.form");
pub const SAVE: Id = FORM.sub("save");
pub const PROVIDER: Id = FORM.sub("provider");
pub const SOURCE: Id = FORM.sub("source");
pub const OP: Id = FORM.sub("op");
pub const START: Id = FORM.sub("start");
pub const NAME: Id = FORM.sub("name");
pub const AGENT: Id = FORM.sub("agent");
pub const FOLDER: Id = FORM.sub("folder");
pub const SECRET: Id = FORM.sub("secret");
pub const FILTER: Id = Id::root("jackin.accounts.filter");

#[derive(Clone, Default, PartialEq, Eq)]
pub struct AccountsState {
    pub list: ListState,
    pub form_open: bool,
    pub started: bool,
    pub editing: Option<String>,
    pub draft_name: String,
    pub masked_input: String,
    pub pending_refresh: Option<String>,
    pub remove_confirmation: Option<String>,
    pub name_input: TextInputState,
    pub folder_input: TextInputState,
    pub secret_input: TextInputState,
    pub provider_list: ListState,
    pub source_list: ListState,
    pub provider_index: u8,
    pub source_index: u8,
    pub selected_op: Option<OpReference>,
    pub op_item: String,
    pub op_stage: u8,
    pub selected_id: Option<String>,
    pub filtering: bool,
    pub filter_query: String,
    pub filter_input: TextInputState,
    pub drawer_open: bool,
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
            .field("filtering", &self.filtering)
            .field("filter_query", &self.filter_query)
            .field("filter_input", &self.filter_input)
            .field("drawer_open", &self.drawer_open)
            .finish()
    }
}

impl AccountsState {
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
        self.filtering = false;
        self.filter_query.clear();
        self.filter_input = TextInputState::default();
        self.drawer_open = false;
    }

    pub const fn close(&mut self) {
        self.form_open = false;
    }

    pub fn form_editing(&self) -> bool {
        self.name_input.is_editing()
            || self.folder_input.is_editing()
            || self.secret_input.is_editing()
    }

    pub fn form_hints(&self) -> &'static str {
        if self.form_editing() {
            "EDIT  Enter Commit  Tab Next field  Esc Revert"
        } else {
            "Tab Next field  Enter Edit / Save  Esc Cancel"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_hints_switch_when_a_field_is_editing() {
        let mut state = AccountsState::default();
        assert!(!state.form_editing());
        assert_eq!(
            state.form_hints(),
            "Tab Next field  Enter Edit / Save  Esc Cancel"
        );
        state.name_input.begin("Team");
        assert!(state.form_editing());
        assert_eq!(
            state.form_hints(),
            "EDIT  Enter Commit  Tab Next field  Esc Revert"
        );
    }
}
