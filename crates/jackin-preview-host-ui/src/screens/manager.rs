use termrock::{Id, ListState};

use jackin_preview_domain::account::AccountId;
use jackin_preview_domain::agent::Agent;
use jackin_preview_domain::workspace::WorkspaceId;
use jackin_preview_sim::world::World;

pub const TREE: Id = Id::root("jackin.manager.tree");
pub const NEW_WORKSPACE: Id = Id::root("jackin.manager.new-workspace");
pub const DETAIL: Id = Id::root("jackin.manager.detail");
pub const LAUNCH: Id = Id::root("jackin.manager.launch");
pub const AGENT_PICKER: Id = LAUNCH.sub("agent-picker");

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ManagerRowKey {
    #[default]
    CurrentDirectory,
    Workspace(WorkspaceId),
    Instance(String),
    NewWorkspace,
}

impl ManagerRowKey {
    pub fn stable_key(&self) -> String {
        match self {
            Self::CurrentDirectory => "current-directory".into(),
            Self::Workspace(id) => format!("workspace:{id}"),
            Self::Instance(id) => format!("instance:{id}"),
            Self::NewWorkspace => "new-workspace".into(),
        }
    }

    pub const fn workspace(&self) -> Option<WorkspaceId> {
        match self {
            Self::Workspace(id) => Some(*id),
            Self::CurrentDirectory | Self::Instance(_) | Self::NewWorkspace => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCandidate {
    pub agent: Agent,
    pub account: Option<AccountId>,
    pub blocked: Option<String>,
}

impl LaunchCandidate {
    pub const fn is_ready(&self) -> bool {
        self.account.is_some() && self.blocked.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManagerState {
    pub list: ListState,
    expanded: Vec<WorkspaceId>,
    rows_revision: u64,
    selected_workspace: Option<WorkspaceId>,
    selected_row: ManagerRowKey,
    detail_open: bool,
}

impl ManagerState {
    pub fn is_expanded(&self, id: WorkspaceId) -> bool {
        self.expanded.contains(&id)
    }

    pub fn toggle(&mut self, id: WorkspaceId) {
        if let Some(index) = self.expanded.iter().position(|value| *value == id) {
            self.expanded.remove(index);
        } else {
            self.expanded.push(id);
        }
        self.rows_revision = self.rows_revision.wrapping_add(1);
    }

    pub fn invalidate_rows(&mut self) {
        self.rows_revision = self.rows_revision.wrapping_add(1);
    }

    pub const fn rows_revision(&self) -> u64 {
        self.rows_revision
    }

    pub fn select(&mut self, id: Option<WorkspaceId>) {
        self.select_row(match id {
            Some(id) => ManagerRowKey::Workspace(id),
            None => ManagerRowKey::CurrentDirectory,
        });
    }

    pub const fn selected(&self) -> Option<WorkspaceId> {
        self.selected_workspace
    }

    pub const fn selected_row(&self) -> &ManagerRowKey {
        &self.selected_row
    }

    pub fn select_row(&mut self, row: ManagerRowKey) {
        self.selected_workspace = row.workspace();
        self.selected_row = row;
    }

    pub fn launch_candidates(
        world: &World,
        workspace: Option<WorkspaceId>,
        role: Option<&str>,
    ) -> Vec<LaunchCandidate> {
        let workspace_ref = workspace.and_then(|id| world.workspace(id));
        world
            .offered_agents(workspace_ref, role)
            .into_iter()
            .map(|(agent, offer)| LaunchCandidate {
                agent,
                account: offer.preselected,
                blocked: offer.blocked,
            })
            .collect()
    }

    pub const fn set_detail_open(&mut self, open: bool) {
        self.detail_open = open;
    }

    pub const fn detail_open(&self) -> bool {
        self.detail_open
    }
}
