//! Pinned repository catalogue and captured Manager `o` effect proofs.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "Source fixture assertions"
)]
use jackin_app::screens::manager_actions::{Effect, RepositoryError, RepositoryTarget};
use jackin_app::{Scenario, world_for};

#[test]
fn every_scenario_keeps_all_source_repository_metadata() {
    let expected = [
        (
            "chainargos/payments-platform",
            "main",
            vec!["main", "feature/settlement-backoff", "release/2026.09"],
            "1 h ago",
        ),
        (
            "chainargos/infra-control-plane",
            "main",
            vec!["main", "sre/node-pools"],
            "3 h ago",
        ),
        (
            "chainargos/release-automation",
            "main",
            vec!["main", "node-22"],
            "2 d ago",
        ),
        (
            "chainargos/customer-portal",
            "develop",
            vec!["develop", "main", "feature/skeletons"],
            "5 h ago",
        ),
        ("chainargos/roles", "main", vec!["main"], "6 d ago"),
        (
            "chainargos/docs",
            "main",
            vec!["main", "gh-pages"],
            "3 d ago",
        ),
        (
            "acme-labs/roles-experimental",
            "next",
            vec!["next"],
            "2 mo ago",
        ),
    ];
    for scenario in Scenario::ALL {
        let world = world_for(scenario);
        assert_eq!(world.github.len(), expected.len());
        for (repo, (name, branch, branches, updated)) in world.github.iter().zip(&expected) {
            assert_eq!(&repo.full_name, name);
            assert_eq!(&repo.default_branch, branch);
            assert_eq!(&repo.branches, branches);
            assert_eq!(&repo.updated, updated);
            assert_eq!(repo.url, format!("https://github.com/{name}"));
        }
    }
}

#[test]
fn explicit_workspace_open_retains_target_through_world_reordering() {
    let mut world = world_for(Scenario::Returning);
    let workspace = world
        .workspaces
        .iter()
        .find(|w| w.name == "release-automation")
        .unwrap()
        .id;
    let target = RepositoryTarget::capture(&world, Some(workspace)).unwrap();
    assert_eq!(
        target.url(),
        "https://github.com/chainargos/release-automation"
    );
    world.workspaces.reverse();
    let instances = world.instances.clone();
    let jobs = world.jobs.clone();
    let now = world.now_ms();
    assert!(
        matches!(target.execute(&world), Effect::RepositoryOpened { url, status } if url == "https://github.com/chainargos/release-automation" && status == "Opened https://github.com/chainargos/release-automation on the host")
    );
    assert_eq!(world.instances, instances);
    assert_eq!(world.jobs, jobs);
    assert_eq!(world.now_ms(), now);
}

#[test]
fn current_directory_requires_real_saved_association() {
    let mut world = world_for(Scenario::Returning);
    let expected = world.cwd_workspace().unwrap().name.clone();
    let target = RepositoryTarget::capture(&world, None).unwrap();
    assert!(
        matches!(target.execute(&world), Effect::RepositoryOpened { url, .. } if url.ends_with(&expected))
    );
    world.cwd = "/not/a/saved/workspace".into();
    assert!(
        matches!(RepositoryTarget::capture(&world, None), Err(error) if error.to_string() == "Select a saved workspace to open its repository")
    );
}

#[test]
fn stale_workspace_catalogue_or_cwd_refuses_repository_effect() {
    for mode in 0..5 {
        let mut world = world_for(Scenario::Returning);
        let id = world.cwd_workspace().unwrap().id;
        let target =
            RepositoryTarget::capture(&world, if mode == 4 { None } else { Some(id) }).unwrap();
        match mode {
            0 => world.workspaces.retain(|w| w.id != id),
            1 => world.workspace_mut(id).unwrap().name.push('!'),
            2 => world.github[0].url = "https://example.invalid/replaced".into(),
            3 => world.github.clear(),
            _ => world.cwd.push('!'),
        }
        assert!(
            matches!(target.execute(&world), Effect::Status(text) if text == "Target changed · nothing changed")
        );
    }
}

#[test]
fn missing_repository_preserves_source_feedback_without_inventing_url() {
    let mut world = world_for(Scenario::Returning);
    let id = world.workspaces[0].id;
    world.workspace_mut(id).unwrap().name = "no-discovered-repository".into();
    assert!(
        matches!(RepositoryTarget::capture(&world, Some(id)), Err(error) if error.to_string() == "No GitHub source for no-discovered-repository")
    );
    assert!(matches!(
        RepositoryTarget::capture(&world, Some(u32::MAX)),
        Err(RepositoryError::NoWorkspace)
    ));
}
