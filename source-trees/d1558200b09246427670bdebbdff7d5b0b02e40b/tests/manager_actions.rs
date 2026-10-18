//! Source-bound Manager operation and stale-target consumer proofs.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "Explicit fixture assertions"
)]
use jackin_app::Scenario;
use jackin_app::{
    domain::instance::{InstanceStatus, RunId},
    screens::manager_actions::{Action, Effect, ManagerActions, SessionChoice, Target},
    sim::world::{Msg, World, world_for},
};

fn running(world: &World) -> String {
    world
        .instances
        .iter()
        .find(|i| i.status.is_live())
        .unwrap()
        .id
        .clone()
}
fn review(
    actions: &mut ManagerActions,
    action: Action,
    target: Target,
    world: &mut World,
) -> jackin_app::screens::manager_actions::Review {
    match actions.request(action, target, world) {
        Effect::Confirm(review) => review,
        other => panic!("expected review, got {other:?}"),
    }
}
fn operation(world: &World) -> u64 {
    world
        .jobs
        .iter()
        .find_map(|job| match job.msg {
            Msg::ManagerOperation { operation } => Some(operation),
            _ => None,
        })
        .unwrap()
}
fn returning() -> World {
    world_for(Scenario::Returning)
}

#[test]
fn stop_is_delayed_and_completion_is_exactly_once() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let target = Target::Instance(id.clone());
    let before = world.instance(&id).unwrap().clone();
    let r = review(&mut actions, Action::Stop, target.clone(), &mut world);
    assert_eq!(world.instance(&id), Some(&before));
    assert!(world.jobs.is_empty());
    actions.confirm(r, "", &mut world);
    let op = operation(&world);
    assert_eq!(actions.busy(&target), Some("stopping…"));
    assert!(matches!(actions.complete(&mut world, op), Effect::None));
    assert!(world.tick(1799).is_empty());
    assert_eq!(world.instance(&id).unwrap().status, InstanceStatus::Running);
    assert_eq!(world.tick(1), vec![Msg::ManagerOperation { operation: op }]);
    assert!(matches!(
        actions.complete(&mut world, op),
        Effect::Status(_)
    ));
    assert_eq!(
        world.instance(&id).unwrap().status,
        if before.is_dirty() {
            InstanceStatus::PreservedDirty
        } else {
            InstanceStatus::RestoreAvailable
        }
    );
    assert!(!world.daemons.contains_key(&id));
    assert_eq!(actions.busy(&target), None);
    let after = world.instance(&id).unwrap().clone();
    assert!(matches!(actions.complete(&mut world, op), Effect::None));
    assert_eq!(world.instance(&id), Some(&after));
}

#[test]
fn purge_requires_exact_ack_and_retains_record_after_delay() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let target = Target::Instance(id.clone());
    let r = review(&mut actions, Action::Purge, target.clone(), &mut world);
    let ack = r.acknowledgement().unwrap().to_owned();
    actions.confirm(r, &format!(" {ack}"), &mut world);
    assert!(world.jobs.is_empty());
    let r = review(&mut actions, Action::Purge, target, &mut world);
    actions.confirm(r, &ack, &mut world);
    let op = operation(&world);
    assert!(world.tick(2199).is_empty());
    world.tick(1);
    actions.complete(&mut world, op);
    assert_eq!(world.instance(&id).unwrap().status, InstanceStatus::Purged);
    assert!(!world.daemons.contains_key(&id));
}

#[test]
fn replaced_removed_or_changed_review_target_cannot_mutate() {
    for mode in 0..4 {
        let mut world = returning();
        let mut actions = ManagerActions::default();
        let id = running(&world);
        let r = review(
            &mut actions,
            Action::Stop,
            Target::Instance(id.clone()),
            &mut world,
        );
        match mode {
            0 => world.instance_mut(&id).unwrap().run_id = RunId::new(999),
            1 => world.instances.retain(|i| i.id != id),
            2 => world.instance_mut(&id).unwrap().uncommitted += 1,
            _ => world.instance_mut(&id).unwrap().role = "other/role".into(),
        }
        let instances = world.instances.clone();
        actions.confirm(r, "", &mut world);
        assert_eq!(world.instances, instances);
        assert!(world.jobs.is_empty());
    }
}

#[test]
fn duplicate_operations_and_stale_completions_do_not_touch_replacement() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let target = Target::Instance(id.clone());
    let r1 = review(&mut actions, Action::Stop, target.clone(), &mut world);
    let r2 = review(&mut actions, Action::Stop, target.clone(), &mut world);
    actions.confirm(r1, "", &mut world);
    actions.confirm(r2, "", &mut world);
    assert_eq!(world.jobs.len(), 1);
    let op = operation(&world);
    world.instance_mut(&id).unwrap().run_id = RunId::new(42);
    world.tick(1800);
    let instances = world.instances.clone();
    let daemon = world.daemons[&id].snapshot();
    actions.complete(&mut world, op);
    assert_eq!(world.instances, instances);
    assert_eq!(world.daemons[&id].snapshot(), daemon);
    assert!(actions.busy(&target).is_none());
}

#[test]
fn delete_detaches_instances_preserving_daemons_and_rejects_changed_config() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = world
        .instances
        .iter()
        .find(|i| i.workspace.is_some())
        .unwrap()
        .workspace
        .unwrap();
    let r = review(
        &mut actions,
        Action::Delete,
        Target::Workspace(id),
        &mut world,
    );
    world.workspace_mut(id).unwrap().name.push('!');
    actions.confirm(r, "", &mut world);
    assert!(world.workspace(id).is_some());
    let r = review(
        &mut actions,
        Action::Delete,
        Target::Workspace(id),
        &mut world,
    );
    let instances = world.instances.clone();
    let daemons: Vec<_> = world.daemons.keys().cloned().collect();
    assert!(
        matches!(actions.confirm(r, "", &mut world), Effect::WorkspaceDeleted { id: removed, .. } if removed == id)
    );
    assert!(world.workspace(id).is_none());
    assert_eq!(world.instances.len(), instances.len());
    assert_eq!(world.daemons.keys().cloned().collect::<Vec<_>>(), daemons);
    for (before, after) in instances.iter().zip(&world.instances) {
        let mut expected = before.clone();
        if expected.workspace == Some(id) {
            expected.workspace = None;
        }
        assert_eq!(&expected, after);
    }
}

#[test]
fn prewarm_has_single_authoritative_busy_state_and_source_delay() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = world.workspaces[2].id;
    let target = Target::Workspace(id);
    actions.request(Action::Prewarm, target.clone(), &mut world);
    actions.request(Action::Prewarm, target.clone(), &mut world);
    assert_eq!(world.jobs.len(), 1);
    assert_eq!(actions.busy(&target), Some("prewarming…"));
    let op = operation(&world);
    assert!(world.tick(2399).is_empty());
    world.tick(1);
    assert!(
        matches!(actions.complete(&mut world, op), Effect::Status(text) if text.contains(&world.workspace(id).unwrap().name))
    );
    assert!(actions.busy(&target).is_none());
}

#[test]
fn restore_rebuilds_actual_daemon_and_running_attach_keeps_it() {
    for status in [
        InstanceStatus::RestoreAvailable,
        InstanceStatus::PreservedDirty,
        InstanceStatus::PreservedUnpushed,
    ] {
        let mut world = returning();
        let mut actions = ManagerActions::default();
        let id = running(&world);
        world.instance_mut(&id).unwrap().status = status;
        world.daemons.remove(&id);
        assert!(
            matches!(actions.request(Action::Reconnect, Target::Instance(id.clone()), &mut world), Effect::Attach { target, status: Some(_) } if target.id == id)
        );
        assert_eq!(world.instance(&id).unwrap().status, InstanceStatus::Running);
        assert_eq!(world.daemons[&id].tabs.len(), 1);
        assert_eq!(world.daemons[&id].panes.len(), 1);
        let snapshot = world.daemons[&id].snapshot();
        assert!(matches!(
            actions.request(Action::Reconnect, Target::Instance(id.clone()), &mut world),
            Effect::Attach { status: None, .. }
        ));
        assert_eq!(world.daemons[&id].snapshot(), snapshot);
    }
}

#[test]
fn session_picker_captures_instance_and_rechecks_lifecycle_and_accounts() {
    let mut world = returning();
    let actions = &mut ManagerActions::default();
    let id = running(&world);
    let Effect::ChooseSession(r) =
        actions.request(Action::NewSession, Target::Instance(id.clone()), &mut world)
    else {
        panic!()
    };
    world.instance_mut(&id).unwrap().run_id = RunId::new(78);
    assert!(matches!(
        actions.session(r, SessionChoice::Shell, &mut world),
        Effect::Status(_)
    ));
    let Effect::ChooseSession(r) =
        actions.request(Action::NewSession, Target::Instance(id.clone()), &mut world)
    else {
        panic!()
    };
    assert!(
        matches!(actions.session(r, SessionChoice::Shell, &mut world), Effect::NewSession { target, choice: SessionChoice::Shell } if target.id == id)
    );
    let Effect::ChooseSession(r) =
        actions.request(Action::NewSession, Target::Instance(id.clone()), &mut world)
    else {
        panic!()
    };
    let agent = world.instance(&id).unwrap().agent;
    assert!(matches!(
        actions.session(
            r,
            SessionChoice::Agent {
                agent,
                account: Some("unknown".into())
            },
            &mut world
        ),
        Effect::Status(_)
    ));
}

#[test]
fn inspection_is_readonly_and_action_inventory_tracks_lifecycle() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let target = Target::Instance(id.clone());
    let instances = world.instances.clone();
    let Effect::Inspect { facts, .. } =
        actions.request(Action::Inspect, target.clone(), &mut world)
    else {
        panic!()
    };
    assert_eq!(world.instances, instances);
    assert!(world.jobs.is_empty());
    assert_eq!(facts.len(), 8);
    assert_eq!(
        facts
            .iter()
            .filter(|f| f.copyable)
            .map(|f| f.label)
            .collect::<Vec<_>>(),
        ["Container", "Run id"]
    );
    for status in [
        InstanceStatus::Running,
        InstanceStatus::CleanExited,
        InstanceStatus::Crashed,
        InstanceStatus::PreservedDirty,
        InstanceStatus::PreservedUnpushed,
        InstanceStatus::RestoreAvailable,
        InstanceStatus::Superseded,
        InstanceStatus::Purged,
        InstanceStatus::FailedSetup,
    ] {
        world.instance_mut(&id).unwrap().status = status;
        assert_eq!(
            ManagerActions::available(Action::Stop, &target, &world),
            status.is_live()
        );
        assert_eq!(
            ManagerActions::available(Action::NewSession, &target, &world),
            status.is_live()
        );
        assert_eq!(
            ManagerActions::available(Action::Reconnect, &target, &world),
            status.reconnectable() && status != InstanceStatus::Crashed
        );
    }
    assert!(matches!(
        actions.request(
            Action::Purge,
            Target::Instance("missing".into()),
            &mut world
        ),
        Effect::None
    ));
}

#[test]
fn launch_scope_is_captured_and_never_falls_back_to_first_workspace() {
    use jackin_app::screens::manager_actions::LaunchTarget;
    let mut world = returning();
    let workspace = world.workspaces[2].clone();
    let role = workspace.roles.default.as_deref().unwrap();
    let target = LaunchTarget::capture(&world, Some(workspace.id), role).unwrap();
    assert_eq!(target.workspace(), Some(workspace.id));
    assert_eq!(target.workdir(), workspace.workdir);
    assert_eq!(target.role(), role);
    world.workspaces.reverse();
    assert!(target.valid(&world));
    world.workspace_mut(workspace.id).unwrap().workdir.push('!');
    assert!(!target.valid(&world));
    assert!(LaunchTarget::capture(&world, Some(u32::MAX), role).is_none());
    assert!(LaunchTarget::capture(&world, None, "missing/role").is_none());
    let target = LaunchTarget::capture(&world, None, role).unwrap();
    assert_eq!(target.workspace(), None);
    assert_eq!(target.workdir(), world.cwd);
    world.cwd.push('!');
    assert!(!target.valid(&world));
}

#[test]
fn refresh_observes_first_failure_after_five_seconds_and_keeps_stale_latch() {
    use jackin_app::sim::world::DaemonHealth;
    let mut world = returning();
    let mut actions = ManagerActions::default();
    world.refresh_fails = true;
    world.last_refresh_secs = world.now_secs();
    assert_eq!(world.daemon_health, DaemonHealth::Healthy);
    world.tick(4999);
    assert!(!actions.refresh(&mut world));
    assert!(world.jobs.is_empty());
    world.tick(1);
    assert!(actions.refresh(&mut world));
    assert_eq!(world.daemon_health, DaemonHealth::Stale);
    assert_eq!(world.tick(0), [Msg::Refreshed { ok: false }]);
    assert!(!actions.refresh(&mut world));
    world.tick(5000);
    assert!(actions.refresh(&mut world));
    assert!(world.jobs.is_empty());
    for instance in &world.instances {
        if instance.status.is_live() {
            assert_eq!(instance.last_seen_secs, world.now_secs());
        }
    }
    world.refresh_fails = false;
    world.tick(5000);
    assert!(actions.refresh(&mut world));
    assert_eq!(world.daemon_health, DaemonHealth::Stale);
    assert!(world.jobs.is_empty());
}

#[test]
fn successful_refresh_preserves_health_and_updates_only_live_seen_times() {
    use jackin_app::sim::world::DaemonHealth;
    let mut world = returning();
    let mut actions = ManagerActions::default();
    world.last_refresh_secs = world.now_secs();
    let before = world.instances.clone();
    world.tick(5000);
    assert!(actions.refresh(&mut world));
    assert_eq!(world.daemon_health, DaemonHealth::Healthy);
    assert!(world.jobs.is_empty());
    for (old, new) in before.iter().zip(&world.instances) {
        assert_eq!(
            new.last_seen_secs,
            if new.status.is_live() {
                world.now_secs()
            } else {
                old.last_seen_secs
            }
        );
    }
}

#[test]
fn reducer_replacement_cannot_recycle_a_queued_completion_token() {
    let mut world = returning();
    let id = running(&world);
    let target = Target::Instance(id.clone());
    let mut first = ManagerActions::default();
    let r = review(&mut first, Action::Stop, target.clone(), &mut world);
    first.confirm(r, "", &mut world);
    let old = operation(&world);
    let mut replacement = ManagerActions::default();
    let r = review(&mut replacement, Action::Stop, target.clone(), &mut world);
    replacement.confirm(r, "", &mut world);
    let ids: Vec<_> = world
        .jobs
        .iter()
        .filter_map(|j| match j.msg {
            Msg::ManagerOperation { operation } => Some(operation),
            _ => None,
        })
        .collect();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1]);
    world.tick(1800);
    assert!(matches!(
        replacement.complete(&mut world, old),
        Effect::None
    ));
    assert_eq!(world.instance(&id).unwrap().status, InstanceStatus::Running);
    replacement.complete(&mut world, ids[1]);
    assert_ne!(world.instance(&id).unwrap().status, InstanceStatus::Running);
}

#[test]
fn workspace_context_changes_invalidate_instance_review_and_session_picker() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let workspace = world.instance(&id).unwrap().workspace.unwrap();
    let r = review(
        &mut actions,
        Action::Purge,
        Target::Instance(id.clone()),
        &mut world,
    );
    let ack = r.acknowledgement().unwrap().to_owned();
    let Effect::ChooseSession(session) =
        actions.request(Action::NewSession, Target::Instance(id.clone()), &mut world)
    else {
        panic!()
    };
    world.workspace_mut(workspace).unwrap().name.push('!');
    actions.confirm(r, &ack, &mut world);
    assert!(world.jobs.is_empty());
    assert!(matches!(
        actions.session(session, SessionChoice::Shell, &mut world),
        Effect::Status(_)
    ));
}

#[test]
fn account_display_adapters_preserve_source_labels_and_unknown_identity() {
    use jackin_app::domain::fixtures::{PrecedenceLevel, ResolvedAccount};
    let world = returning();
    let labels = [
        (PrecedenceLevel::Session, "session choice"),
        (PrecedenceLevel::Role, "Role choice"),
        (PrecedenceLevel::Workspace, "Workspace choice"),
        (PrecedenceLevel::Global, "provider default"),
        (PrecedenceLevel::Discovered, "discovered on host"),
    ];
    for (level, label) in labels {
        assert_eq!(level.label(), label);
        assert_eq!(
            ResolvedAccount {
                account: None,
                level: Some(level),
                reason: String::new()
            }
            .level_label(),
            label
        );
    }
    let mut resolved = ResolvedAccount {
        account: None,
        level: None,
        reason: "unchanged".into(),
    };
    assert_eq!(resolved.label(&world.accounts), "no account");
    assert_eq!(resolved.level_label(), "no account");
    resolved.account = Some("unknown-id".into());
    assert_eq!(resolved.label(&world.accounts), "unknown-id");
    assert_eq!(resolved.reason, "unchanged");
    let instance = world
        .instances
        .iter()
        .find(|i| !i.accounts.is_empty())
        .unwrap();
    let id = instance.accounts[0].clone();
    resolved.account = Some(id.clone());
    assert_eq!(
        resolved.label(&world.accounts),
        world.accounts.get(&id).unwrap().title()
    );
}

#[test]
fn every_nonrunning_reconnect_outcome_is_source_bound_and_nonmutating() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    for (status, message) in [
        (InstanceStatus::Crashed, Some("crashed (exit 137)")),
        (
            InstanceStatus::FailedSetup,
            Some("never reached the Capsule"),
        ),
        (InstanceStatus::CleanExited, Some("exited cleanly")),
        (InstanceStatus::Purged, None),
        (InstanceStatus::Superseded, None),
    ] {
        world.instance_mut(&id).unwrap().status = status;
        let before = world.instances.clone();
        let effect = actions.request(Action::Reconnect, Target::Instance(id.clone()), &mut world);
        match message {
            Some(expected) => {
                assert!(
                    matches!(effect, Effect::Status(text) | Effect::Error(text) if text.contains(expected))
                );
            }
            None => assert!(matches!(effect, Effect::None)),
        }
        assert_eq!(world.instances, before);
        assert!(world.jobs.is_empty());
    }
}

#[test]
fn session_account_choice_is_real_and_revalidated_after_picker_open() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let instance = world.instance(&id).unwrap();
    let agent = instance.agent;
    let account = world
        .offer_for(
            agent,
            instance.workspace.and_then(|id| world.workspace(id)),
            Some(&instance.role),
        )
        .accounts
        .into_iter()
        .next()
        .unwrap();
    let Effect::ChooseSession(r) =
        actions.request(Action::NewSession, Target::Instance(id.clone()), &mut world)
    else {
        panic!()
    };
    let choice = SessionChoice::Agent {
        agent,
        account: Some(account.clone()),
    };
    assert!(
        matches!(actions.session(r, choice.clone(), &mut world), Effect::NewSession { choice: emitted, .. } if emitted == choice)
    );
    let Effect::ChooseSession(r) =
        actions.request(Action::NewSession, Target::Instance(id), &mut world)
    else {
        panic!()
    };
    world.accounts.get_mut(&account).unwrap().enabled = false;
    assert!(matches!(
        actions.session(r, choice, &mut world),
        Effect::Status(_)
    ));
}

#[test]
fn cancelling_review_has_no_effect_and_purge_tones_preserve_risk_contract() {
    use jackin_app::screens::manager_actions::FactTone;
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let before = world.instances.clone();
    let r = review(
        &mut actions,
        Action::Purge,
        Target::Instance(id.clone()),
        &mut world,
    );
    assert_eq!(
        r.facts().iter().find(|f| f.label == "Action").unwrap().tone,
        FactTone::Error
    );
    assert_eq!(
        r.facts().iter().find(|f| f.label == "Risk").unwrap().tone,
        FactTone::Warning
    );
    drop(r);
    assert!(world.jobs.is_empty());
    assert_eq!(world.instances, before);
    assert!(actions.busy(&Target::Instance(id)).is_none());
}

#[test]
fn cloned_review_cannot_replay_after_stop_then_restore() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let target = Target::Instance(id.clone());
    let r = review(&mut actions, Action::Stop, target.clone(), &mut world);
    let replay = r.clone();
    actions.confirm(r, "", &mut world);
    let op = operation(&world);
    world.tick(1800);
    actions.complete(&mut world, op);
    actions.request(Action::Reconnect, target, &mut world);
    assert_eq!(world.instance(&id).unwrap().status, InstanceStatus::Running);
    actions.confirm(replay, "", &mut world);
    assert!(world.jobs.is_empty());
}

#[test]
fn cloned_world_and_reducer_consume_review_independently() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let r = review(&mut actions, Action::Stop, Target::Instance(id), &mut world);
    let mut other_world = world.clone();
    let mut other_actions = actions.clone();
    let other_review = r.clone();
    actions.confirm(r, "", &mut world);
    other_actions.confirm(other_review, "", &mut other_world);
    assert_eq!(world.jobs, other_world.jobs);
    assert_eq!(world.jobs.len(), 1);
}

#[test]
fn cloned_session_choice_cannot_create_two_sessions_in_same_world() {
    let mut world = returning();
    let mut actions = ManagerActions::default();
    let id = running(&world);
    let Effect::ChooseSession(r) =
        actions.request(Action::NewSession, Target::Instance(id), &mut world)
    else {
        panic!()
    };
    let replay = r.clone();
    assert!(matches!(
        actions.session(r, SessionChoice::Shell, &mut world),
        Effect::NewSession { .. }
    ));
    assert!(matches!(
        actions.session(replay, SessionChoice::Shell, &mut world),
        Effect::Status(_)
    ));
}
