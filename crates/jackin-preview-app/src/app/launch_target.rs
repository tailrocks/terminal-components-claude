use crate::domain::workspace::{Workspace, WorkspaceId};
use crate::screens::manager::ManagerRowKey;
use crate::sim::world::World;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LaunchContextError {
    TargetChanged,
    RoleUnavailable,
    AccountUnavailable,
}

impl LaunchContextError {
    pub(super) const fn status(&self) -> &'static str {
        match self {
            Self::TargetChanged => "Launch target changed; select it again",
            Self::RoleUnavailable => "Selected role is no longer available",
            Self::AccountUnavailable => "Selected agent account is no longer available",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LaunchTarget {
    Workspace(Workspace),
    CurrentDirectory {
        raw_cwd: String,
        workspace: Option<Workspace>,
    },
    Seeded {
        workspace: Option<Workspace>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LaunchContext {
    target: LaunchTarget,
    role: String,
}

impl LaunchContext {
    pub(super) fn for_manager(world: &World, row: &ManagerRowKey, role: &str) -> Option<Self> {
        let target = match row {
            ManagerRowKey::Workspace(id) => LaunchTarget::Workspace(world.workspace(*id)?.clone()),
            ManagerRowKey::CurrentDirectory => LaunchTarget::CurrentDirectory {
                raw_cwd: world.cwd.clone(),
                workspace: world.cwd_workspace().cloned(),
            },
            ManagerRowKey::Instance(_) | ManagerRowKey::NewWorkspace => return None,
        };
        Some(Self {
            target,
            role: role.to_owned(),
        })
    }

    pub(super) fn seeded(world: &World, role: &str) -> Self {
        Self {
            target: LaunchTarget::Seeded {
                workspace: world.workspaces.first().cloned(),
            },
            role: role.to_owned(),
        }
    }

    pub(super) fn role(&self) -> &str {
        &self.role
    }

    pub(super) fn workspace(&self) -> Option<&Workspace> {
        match &self.target {
            LaunchTarget::Workspace(workspace) => Some(workspace),
            LaunchTarget::CurrentDirectory { workspace, .. }
            | LaunchTarget::Seeded { workspace } => workspace.as_ref(),
        }
    }

    pub(super) fn workspace_id(&self) -> Option<WorkspaceId> {
        self.workspace().map(|workspace| workspace.id)
    }

    pub(super) fn workdir(&self) -> String {
        match &self.target {
            LaunchTarget::Workspace(workspace) => workspace.workdir.clone(),
            LaunchTarget::CurrentDirectory {
                raw_cwd,
                workspace: None,
            } => format!("/workspace/{}", path_basename(raw_cwd)),
            LaunchTarget::CurrentDirectory {
                workspace: Some(workspace),
                ..
            }
            | LaunchTarget::Seeded {
                workspace: Some(workspace),
            } => workspace.workdir.clone(),
            LaunchTarget::Seeded { workspace: None } => String::new(),
        }
    }

    pub(super) fn container(&self) -> String {
        match &self.target {
            LaunchTarget::Seeded { .. } => "jackin-payments-platform".to_owned(),
            LaunchTarget::Workspace(workspace) => container_for(&workspace.workdir),
            LaunchTarget::CurrentDirectory {
                raw_cwd,
                workspace: None,
            } => container_for(raw_cwd),
            LaunchTarget::CurrentDirectory {
                workspace: Some(workspace),
                ..
            } => container_for(&workspace.workdir),
        }
    }

    pub(super) fn scope_label(&self) -> String {
        match &self.target {
            LaunchTarget::Workspace(workspace) => {
                format!("{} › {}", workspace.name, self.role)
            }
            LaunchTarget::CurrentDirectory { raw_cwd, .. } => format!(
                "Current directory · {} › {}",
                path_basename(raw_cwd),
                self.role
            ),
            LaunchTarget::Seeded { workspace } => format!(
                "{} › {}",
                workspace
                    .as_ref()
                    .map_or("Current directory", |saved| saved.name.as_str()),
                self.role
            ),
        }
    }

    pub(super) fn validate(&self, world: &World) -> Result<(), LaunchContextError> {
        if !world.roles.iter().any(|role| role.full_name() == self.role) {
            return Err(LaunchContextError::RoleUnavailable);
        }
        let unchanged = match &self.target {
            LaunchTarget::Workspace(workspace) => world.workspace(workspace.id) == Some(workspace),
            LaunchTarget::CurrentDirectory { raw_cwd, workspace } => {
                world.cwd == *raw_cwd && world.cwd_workspace() == workspace.as_ref()
            }
            LaunchTarget::Seeded { workspace } => match workspace {
                Some(workspace) => world.workspace(workspace.id) == Some(workspace),
                None => world.workspaces.is_empty(),
            },
        };
        unchanged
            .then_some(())
            .ok_or(LaunchContextError::TargetChanged)
    }
}

fn path_basename(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .find(|part| !part.is_empty())
        .filter(|part| *part != "." && *part != "..")
        .unwrap_or("current-directory")
        .to_owned()
}

fn container_for(path: &str) -> String {
    let mut slug = String::new();
    for character in path_basename(path).chars().take(48) {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "jackin-current-directory".to_owned()
    } else {
        format!("jackin-{slug}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::Scenario;
    use crate::sim::world::world_for;

    #[test]
    fn current_directory_context_keeps_raw_path_and_exact_mount_across_reorder() {
        let mut world = world_for(Scenario::Returning);
        let target = world
            .workspaces
            .get(1)
            .expect("non-first workspace")
            .clone();
        let cwd = "/tmp/customer-portal";
        let mut mounted = target.clone();
        mounted.workdir = "/workspace/configured-repo".to_owned();
        mounted.mounts = vec![crate::domain::workspace::Mount::host(
            cwd,
            "/workspace/configured-repo",
        )];
        world.workspaces[1] = mounted.clone();
        world.cwd = cwd.to_owned();
        let context = LaunchContext::for_manager(
            &world,
            &ManagerRowKey::CurrentDirectory,
            "chainargos/the-architect",
        )
        .expect("CurrentDirectory launch context");
        assert_eq!(context.workspace(), Some(&mounted));
        assert_eq!(context.workspace_id(), Some(mounted.id));
        assert_eq!(context.workdir(), mounted.workdir);
        assert_eq!(context.container(), container_for(&mounted.workdir));
        assert_eq!(
            context.scope_label(),
            "Current directory · customer-portal › chainargos/the-architect"
        );
        assert!(context.validate(&world).is_ok());

        world.workspaces.swap(0, 1);
        assert!(context.validate(&world).is_ok());
        world.cwd.push_str("/nested");
        assert_eq!(
            context.validate(&world),
            Err(LaunchContextError::TargetChanged)
        );
    }

    #[test]
    fn context_preserves_the_exact_selected_role() {
        let world = world_for(Scenario::Returning);
        let role = "chainargos/backend";
        let context = LaunchContext::for_manager(&world, &ManagerRowKey::CurrentDirectory, role)
            .expect("CurrentDirectory launch context");

        assert_eq!(context.role(), role);
        assert!(context.validate(&world).is_ok());
    }

    #[test]
    fn unsaved_current_directory_uses_global_context_and_safe_basename() {
        let mut world = world_for(Scenario::Returning);
        world.cwd = "/".to_owned();
        let context = LaunchContext::for_manager(
            &world,
            &ManagerRowKey::CurrentDirectory,
            "chainargos/the-architect",
        )
        .expect("CurrentDirectory launch context");
        assert_eq!(context.workspace(), None);
        assert_eq!(context.workdir(), "/workspace/current-directory");
        assert_eq!(context.container(), "jackin-current-directory");
        assert!(context.validate(&world).is_ok());
    }
}
