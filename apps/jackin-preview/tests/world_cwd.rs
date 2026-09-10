//! Current directory is domain truth, never inferred from the selected row.
use jackin_app::domain::workspace::Mount;
use jackin_app::{Scenario, world_for};

#[test]
fn source_initial_directory_can_be_saved_or_unsaved() {
    for scenario in Scenario::ALL {
        let world = world_for(scenario);
        assert_eq!(world.cwd, "/Users/alexey/src/payments-platform");
        assert_eq!(
            world.cwd_workspace().map(|w| w.id),
            if scenario == Scenario::FirstUse {
                None
            } else {
                Some(1)
            }
        );
    }
}

#[test]
fn mount_identity_is_exact_and_reconciles_domain_mutations() {
    let mut world = world_for(Scenario::Returning);
    if let Some(workspace) = world.workspace_mut(1) {
        workspace.mounts = vec![Mount::host(
            "~/src/payments-platform",
            "/workspace/payments-platform",
        )];
        workspace.name = "renamed".into();
    }
    assert_eq!(
        world.cwd_workspace().map(|w| w.name.as_str()),
        Some("renamed")
    );
    world.workspaces.swap(0, 1);
    assert_eq!(world.cwd_workspace().map(|w| w.id), Some(1));
    world.cwd.push_str("/child");
    assert_eq!(world.cwd_workspace().map(|w| w.id), None);
    world.cwd = world.home.clone();
    assert_eq!(world.cwd_workspace().map(|w| w.id), None);
    world.cwd = "/Users/alexey/src/payments-platform".into();
    world.workspaces.retain(|w| w.id != 1);
    assert_eq!(world.cwd_workspace().map(|w| w.id), None);
}

#[test]
fn host_cwd_and_container_destination_are_distinct() {
    let mut world = world_for(Scenario::Returning);
    if let Some(workspace) = world.workspace_mut(1) {
        workspace.mounts = vec![Mount::host(
            "/other/source",
            "/Users/alexey/src/payments-platform",
        )];
    }
    assert_eq!(world.cwd_workspace().map(|w| w.id), None);
    world.cwd = "/other/source".into();
    assert_eq!(world.cwd_workspace().map(|w| w.id), Some(1));
}
