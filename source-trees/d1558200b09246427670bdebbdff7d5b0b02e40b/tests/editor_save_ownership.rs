//! Captured editor writes through the production editor and deterministic World APIs.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "Explicit fixture assertions"
)]
use jackin_app::domain::workspace_save::{SaveError, SaveResult};
use jackin_app::screens::editor::{EditorState, PendingWorkspace};
use jackin_app::sim::world::Msg;
use jackin_app::{Scenario, world_for};

fn loaded(world: &jackin_app::sim::world::World, index: usize) -> EditorState {
    let mut editor = EditorState::default();
    editor.load_workspace(&world.workspaces[index]);
    editor.pending.name.push_str("-reviewed");
    editor.mark_dirty();
    assert!(editor.open_preview());
    editor
}

#[test]
fn loaded_third_workspace_owns_target_and_nine_hundred_ms_payload() {
    let mut world = world_for(Scenario::Returning);
    let first = world.workspaces[0].clone();
    let mut editor = loaded(&world, 2);
    let expected = editor.pending.clone();
    let ticket = editor.begin_save(&mut world).unwrap();
    assert_eq!(ticket.workspace, world.workspaces[2].id);
    assert!(editor.dirty);
    assert!(editor.is_saving());
    assert!(!world.saved);
    assert!(matches!(
        world.complete_editor_save(ticket.operation),
        SaveResult::Ignored
    ));
    assert!(world.tick(899).is_empty());
    assert!(!world.saved);
    assert_eq!(
        world.tick(1),
        vec![Msg::EditorSaveCompleted {
            operation: ticket.operation
        }]
    );
    let result = world.complete_editor_save(ticket.operation);
    assert!(editor.settle_save(&result));
    assert!(!editor.dirty);
    assert!(!editor.is_saving());
    assert!(world.saved);
    assert_eq!(
        world.workspace(ticket.workspace).unwrap().name,
        expected.name
    );
    assert_eq!(world.workspace(first.id), Some(&first));
    assert!(matches!(
        world.complete_editor_save(ticket.operation),
        SaveResult::Ignored
    ));
}

#[test]
fn newer_draft_is_not_written_or_cleared_by_old_completion() {
    let mut world = world_for(Scenario::Returning);
    let mut editor = loaded(&world, 1);
    let approved = editor.pending.name.clone();
    let ticket = editor.begin_save(&mut world).unwrap();
    editor.pending.name = "newer-unreviewed".into();
    world.tick(900);
    let result = world.complete_editor_save(ticket.operation);
    assert_eq!(world.workspace(ticket.workspace).unwrap().name, approved);
    assert!(!editor.settle_save(&result));
    assert_eq!(editor.pending.name, "newer-unreviewed");
    assert!(editor.dirty);
    assert!(editor.open_preview());
    let next = editor.begin_save(&mut world).unwrap();
    world.tick(900);
    let result = world.complete_editor_save(next.operation);
    assert!(editor.settle_save(&result));
    assert_eq!(
        world.workspace(next.workspace).unwrap().name,
        "newer-unreviewed"
    );
}

#[test]
fn changed_preview_and_duplicate_admission_never_replace_payload() {
    let mut world = world_for(Scenario::Returning);
    let mut editor = loaded(&world, 0);
    editor.pending.name.push('!');
    assert_eq!(editor.begin_save(&mut world), Err(SaveError::ChangedReview));
    assert!(world.jobs.is_empty());
    assert!(editor.open_preview());
    editor.begin_save(&mut world).unwrap();
    assert_eq!(editor.begin_save(&mut world), Err(SaveError::Busy));
    assert_eq!(world.jobs.len(), 1);
}

#[test]
fn deleted_replaced_and_changed_targets_refuse_without_resurrection() {
    for mode in 0..3 {
        let mut world = world_for(Scenario::Returning);
        let mut editor = loaded(&world, 0);
        let ticket = editor.begin_save(&mut world).unwrap();
        match mode {
            0 => world.workspaces.retain(|w| w.id != ticket.workspace),
            1 => world.workspace_mut(ticket.workspace).unwrap().workdir = "/replacement".into(),
            _ => world.workspace_mut(ticket.workspace).unwrap().keep_awake ^= true,
        }
        let before = world.workspaces.clone();
        world.tick(900);
        let result = world.complete_editor_save(ticket.operation);
        assert!(matches!(result, SaveResult::Stale(_)));
        assert!(!editor.settle_save(&result));
        assert_eq!(world.workspaces, before);
        assert!(editor.dirty);
        assert!(!world.saved);
    }
}

#[test]
fn editor_replacement_cannot_steal_or_drop_approved_world_write() {
    let mut world = world_for(Scenario::Returning);
    let mut editor = loaded(&world, 0);
    let approved = editor.pending.name.clone();
    let ticket = editor.begin_save(&mut world).unwrap();
    editor.load_workspace(&world.workspaces[2]);
    let replacement = editor.pending.clone();
    world.tick(900);
    let result = world.complete_editor_save(ticket.operation);
    assert!(!editor.settle_save(&result));
    assert_eq!(editor.pending, replacement);
    assert_eq!(world.workspace(ticket.workspace).unwrap().name, approved);
}

#[test]
fn source_new_identity_and_one_shot_failure_retry_are_preserved() {
    let mut world = world_for(Scenario::HardCases);
    assert_eq!(world.next_workspace_id, 100);
    assert!(world.save_fails_once);
    let mut editor = EditorState::default();
    let pending = PendingWorkspace {
        name: "new-reviewed".into(),
        ..PendingWorkspace::default()
    };
    editor.load_new(pending);
    assert!(editor.open_preview());
    let ticket = editor.begin_save(&mut world).unwrap();
    assert_eq!(ticket.workspace, 100);
    assert!(!world.save_fails_once);
    world.tick(900);
    let result = world.complete_editor_save(ticket.operation);
    assert!(matches!(result, SaveResult::Failed(_)));
    assert!(!editor.settle_save(&result));
    assert!(world.workspace(100).is_none());
    assert_eq!(world.next_workspace_id, 100);
    assert!(!world.saved);
    assert!(editor.dirty);
    assert!(editor.open_preview());
    let retry = editor.begin_save(&mut world).unwrap();
    assert_eq!(retry.workspace, 100);
    world.tick(900);
    let result = world.complete_editor_save(retry.operation);
    assert!(editor.settle_save(&result));
    assert_eq!(world.workspace(100).unwrap().name, "new-reviewed");
    assert_eq!(world.next_workspace_id, 101);
    assert!(!editor.is_create());
}

#[test]
fn new_save_reservations_do_not_collide_and_external_insert_is_not_overwritten() {
    let mut world = world_for(Scenario::Returning);
    let mut first = EditorState::default();
    first.load_new(PendingWorkspace::default());
    first.open_preview();
    let a = first.begin_save(&mut world).unwrap();
    let mut second = EditorState::default();
    second.load_new(PendingWorkspace::default());
    second.open_preview();
    let b = second.begin_save(&mut world).unwrap();
    assert_eq!((a.workspace, b.workspace), (100, 101));
    let external = jackin_app::domain::workspace::Workspace::new(100, "external", "/external");
    world.workspaces.push(external.clone());
    world.tick(900);
    assert!(matches!(
        world.complete_editor_save(a.operation),
        SaveResult::Stale(_)
    ));
    assert_eq!(world.workspace(100), Some(&external));
    assert!(matches!(
        world.complete_editor_save(b.operation),
        SaveResult::Saved { .. }
    ));
    assert_eq!(world.next_workspace_id, 102);
}

#[test]
fn canonical_role_payload_survives_save_and_cloned_world_is_independent() {
    let mut world = world_for(Scenario::Returning);
    let mut editor = loaded(&world, 0);
    editor.pending.roles.default = Some("chainargos/svc-010".into());
    editor.mark_dirty();
    editor.open_preview();
    let ticket = editor.begin_save(&mut world).unwrap();
    let mut cloned_world = world.clone();
    let mut cloned_editor = editor.clone();
    world.tick(900);
    let result = world.complete_editor_save(ticket.operation);
    assert!(editor.settle_save(&result));
    assert_eq!(
        world
            .workspace(ticket.workspace)
            .unwrap()
            .roles
            .default
            .as_deref(),
        Some("chainargos/svc-010")
    );
    assert!(!cloned_world.saved);
    cloned_world.tick(900);
    let result = cloned_world.complete_editor_save(ticket.operation);
    assert!(cloned_editor.settle_save(&result));
    assert_eq!(world.workspaces, cloned_world.workspaces);
}

#[test]
fn generic_or_unissued_results_cannot_authorize_editor_write() {
    let mut world = world_for(Scenario::Returning);
    let before = world.workspaces.clone();
    world.schedule(
        0,
        Msg::WorkspaceSaved {
            id: before[0].id,
            ok: true,
        },
    );
    world.tick(0);
    assert!(matches!(
        world.complete_editor_save(999),
        SaveResult::Ignored
    ));
    assert_eq!(world.workspaces, before);
    assert!(!world.saved);
}
