//! Manager route state and registered control ids.
//!
//! The manager owns selection semantics while `tui-next::List` owns focus and
//! painting.  Keeping the ids here gives the route and its tests one stable
//! vocabulary without copying a generic widget implementation into the app.

use junie_tui::{Id, ListState};

use crate::domain::workspace::WorkspaceId;

/// Manager tree control.
pub const TREE: Id = Id::root("jackin.manager.tree");
/// New-workspace action.
pub const NEW_WORKSPACE: Id = Id::root("jackin.manager.new-workspace");
/// Instance detail panel.
pub const DETAIL: Id = Id::root("jackin.manager.detail");

/// State owned by the manager route.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManagerState {
    /// Public-list selection state owned by this route.
    pub list: ListState,
    expanded: Vec<WorkspaceId>,
    selected_workspace: Option<WorkspaceId>,
    detail_open: bool,
}

impl ManagerState {
    /// Whether a workspace tree node is expanded.
    pub fn is_expanded(&self, id: WorkspaceId) -> bool {
        self.expanded.contains(&id)
    }

    /// Toggle one workspace node.
    pub fn toggle(&mut self, id: WorkspaceId) {
        if let Some(index) = self.expanded.iter().position(|value| *value == id) {
            self.expanded.remove(index);
        } else {
            self.expanded.push(id);
        }
    }

    /// Select a workspace for detail inspection.
    pub const fn select(&mut self, id: Option<WorkspaceId>) {
        self.selected_workspace = id;
    }

    /// Selected workspace, if any.
    pub const fn selected(&self) -> Option<WorkspaceId> {
        self.selected_workspace
    }

    /// Open or close the detail projection.
    pub const fn set_detail_open(&mut self, open: bool) {
        self.detail_open = open;
    }

    /// Whether the detail projection is visible.
    pub const fn detail_open(&self) -> bool {
        self.detail_open
    }
}
