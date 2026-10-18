#![doc = r" Manager route state and registered control ids."]
#![doc = r""]
#![doc = r" The manager owns selection semantics while `tui-next::List` owns focus and"]
#![doc = r" painting.  Keeping the ids here gives the route and its tests one stable"]
#![doc = r" vocabulary without copying a generic widget implementation into the app."]

use termrock::{Id, ListState};

use jackin_preview_domain::account::AccountId;
use jackin_preview_domain::agent::Agent;
use jackin_preview_domain::workspace::WorkspaceId;
use jackin_preview_sim::world::World;

#[doc = r" Manager tree control."]
pub const TREE: Id = Id::root("jackin.manager.tree");
#[doc = r" New-workspace action."]
pub const NEW_WORKSPACE: Id = Id::root("jackin.manager.new-workspace");
#[doc = r" Instance detail panel."]
pub const DETAIL: Id = Id::root("jackin.manager.detail");
#[doc = r" Launch action owned by the manager."]
pub const LAUNCH: Id = Id::root("jackin.manager.launch");
#[doc = r" Agent picker opened by the launch action."]
pub const AGENT_PICKER: Id = LAUNCH.sub("agent-picker");

#[doc = r" Stable identity for a row in the manager tree."]
#[doc = r""]
#[doc = r" The visible row text is deliberately not used as identity: workspace and"]
#[doc = r" instance labels can change while a cursor is still pointing at the same"]
#[doc = r" durable object."]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ManagerRowKey {
    #[doc = r" The unsaved current directory row."]
    #[default]
    CurrentDirectory,
    #[doc = r" A saved workspace row."]
    Workspace(WorkspaceId),
    #[doc = r" A persisted instance child row."]
    Instance(String),
    #[doc = r" The create-workspace action row."]
    NewWorkspace,
}

impl ManagerRowKey {
    #[doc = r" Return a stable, non-display key suitable for a keyed list item."]
    pub fn stable_key(&self) -> String {
        match self {
            Self::CurrentDirectory => "current-directory".into(),
            Self::Workspace(id) => format!("workspace:{id}"),
            Self::Instance(id) => format!("instance:{id}"),
            Self::NewWorkspace => "new-workspace".into(),
        }
    }

    #[doc = r" Return the workspace represented by this row, if any."]
    pub const fn workspace(&self) -> Option<WorkspaceId> {
        match self {
            Self::Workspace(id) => Some(*id),
            Self::CurrentDirectory | Self::Instance(_) | Self::NewWorkspace => None,
        }
    }
}

#[doc = r" One agent offered by the manager's launch picker."]
#[doc = r""]
#[doc = r" Candidates come from [`World::offered_agents`].  That source filters out"]
#[doc = r" agents with no configured account; a configured but unusable account stays"]
#[doc = r" represented as a blocked candidate so the operator can see why launch is"]
#[doc = r" unavailable."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCandidate {
    #[doc = r" Runtime selected by the candidate."]
    pub agent: Agent,
    #[doc = r" Preselected ready account, when one exists."]
    pub account: Option<AccountId>,
    #[doc = r" Human-readable reason when the configured account is unusable."]
    pub blocked: Option<String>,
}

impl LaunchCandidate {
    #[doc = r" Whether this candidate can start a session immediately."]
    pub const fn is_ready(&self) -> bool {
        self.account.is_some() && self.blocked.is_none()
    }
}

#[doc = r" State owned by the manager route."]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManagerState {
    #[doc = r" Public-list selection state owned by this route."]
    pub list: ListState,
    expanded: Vec<WorkspaceId>,
    #[doc = r" Monotonic key for derived row projections."]
    rows_revision: u64,
    selected_workspace: Option<WorkspaceId>,
    selected_row: ManagerRowKey,
    detail_open: bool,
}

impl ManagerState {
    #[doc = r" Whether a workspace tree node is expanded."]
    pub fn is_expanded(&self, id: WorkspaceId) -> bool {
        self.expanded.contains(&id)
    }

    #[doc = r" Toggle one workspace node."]
    pub fn toggle(&mut self, id: WorkspaceId) {
        if let Some(index) = self.expanded.iter().position(|value| *value == id) {
            self.expanded.remove(index);
        } else {
            self.expanded.push(id);
        }
        self.rows_revision = self.rows_revision.wrapping_add(1);
    }

    #[doc = r" Invalidate the derived row projection after a world refresh."]
    pub fn invalidate_rows(&mut self) {
        self.rows_revision = self.rows_revision.wrapping_add(1);
    }

    #[doc = r" Revision of the expanded-row projection."]
    pub const fn rows_revision(&self) -> u64 {
        self.rows_revision
    }

    #[doc = r" Select a workspace for detail inspection."]
    pub fn select(&mut self, id: Option<WorkspaceId>) {
        self.select_row(match id {
            Some(id) => ManagerRowKey::Workspace(id),
            None => ManagerRowKey::CurrentDirectory,
        });
    }

    #[doc = r" Selected workspace, if any."]
    pub const fn selected(&self) -> Option<WorkspaceId> {
        self.selected_workspace
    }

    #[doc = r" Current stable tree-row selection."]
    pub const fn selected_row(&self) -> &ManagerRowKey {
        &self.selected_row
    }

    #[doc = r" Select a tree row while keeping the workspace projection in sync."]
    pub fn select_row(&mut self, row: ManagerRowKey) {
        self.selected_workspace = row.workspace();
        self.selected_row = row;
    }

    #[doc = r" Build launch candidates for the selected workspace scope."]
    #[doc = r""]
    #[doc = r" This is the one manager-owned path into launch-agent discovery. It"]
    #[doc = r" preserves the world's configured/unconfigured distinction instead of"]
    #[doc = r" rebuilding a positional list from all supported [`Agent`] values."]
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

    #[doc = r" Open or close the detail projection."]
    pub const fn set_detail_open(&mut self, open: bool) {
        self.detail_open = open;
    }

    #[doc = r" Whether the detail projection is visible."]
    pub const fn detail_open(&self) -> bool {
        self.detail_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jackin_preview_domain::account::{Account, CredentialSource, DetectedKind, Lifecycle};
    use jackin_preview_domain::agent::Provider;
    use jackin_preview_domain::scenario::Scenario;
    use jackin_preview_sim::world::world_for;

    #[test]
    fn tree_selection_keeps_stable_row_identity() {
        let mut state = ManagerState::default();
        assert_eq!(state.selected_row(), &ManagerRowKey::CurrentDirectory);

        state.select_row(ManagerRowKey::Workspace(7));
        assert_eq!(state.selected(), Some(7));
        assert_eq!(state.selected_row(), &ManagerRowKey::Workspace(7));
        assert_eq!(state.selected_row().stable_key(), "workspace:7");

        state.select_row(ManagerRowKey::Instance("run-7".into()));
        assert_eq!(state.selected(), None);
        assert_eq!(state.selected_row().stable_key(), "instance:run-7");
    }

    #[test]
    fn launch_candidates_omit_unconfigured_agents() {
        let mut world = world_for(Scenario::FirstUse);
        let mut account = Account::registered(
            "acct-only",
            "Only",
            Provider::Anthropic,
            CredentialSource::LocalFolder {
                path: "~/.claude".into(),
                detected: DetectedKind::ClaudeOAuthProfile,
            },
        );
        account.default_for_provider = true;
        world.accounts.insert(account);

        let candidates = ManagerState::launch_candidates(&world, None, None);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].agent, Agent::ClaudeCode);
        assert!(candidates[0].blocked.is_some());
        assert!(!candidates[0].is_ready());
    }

    #[test]
    fn launch_candidates_preserve_ready_account_selection() {
        let mut world = world_for(Scenario::FirstUse);
        let mut account = Account::registered(
            "acct-ready",
            "Ready",
            Provider::Anthropic,
            CredentialSource::LocalFolder {
                path: "~/.claude".into(),
                detected: DetectedKind::ClaudeOAuthProfile,
            },
        );
        account.lifecycle = Lifecycle::Available;
        account.default_for_provider = true;
        world.accounts.insert(account);

        let candidates = ManagerState::launch_candidates(&world, None, None);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].agent, Agent::ClaudeCode);
        assert_eq!(candidates[0].account.as_deref(), Some("acct-ready"));
        assert_eq!(candidates[0].blocked, None);
        assert!(candidates[0].is_ready());
    }
}
