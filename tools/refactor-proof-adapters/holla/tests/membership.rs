#![allow(
    missing_docs,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! Exact membership: every register row expands, every source site maps,
//! every stage contribution is nonempty, and HO-BASE fixtures follow the
//! oracle world order.

use oracle_holla::branches::NAMED_BRANCHES;
use oracle_holla::expansion::{Action, expand_scenarios};
use oracle_holla::resize_map::native_resize_map;
use oracle_holla::stages::{
    audit_row_count, contributions, shell_contribution_count, shell_frame_contribution_count,
};
use oracle_holla::worlds::{HO_BASE_IDS, ORACLE_WORLDS};

#[test]
fn every_ho_register_row_expands() {
    let programs = expand_scenarios();
    assert_eq!(programs.len(), 135);
    assert_eq!(programs[0].id, "HO-BASE-01");
    assert_eq!(programs[0].fixture, "first-use");
    assert!(
        programs[0]
            .actions
            .iter()
            .any(|action| matches!(action, Action::New { tick: 40, .. }))
    );
    assert_eq!(programs[33].id, "HO-BASE-34");
    assert_eq!(programs[33].fixture, "parity-platforms-linux");
    let ids: Vec<_> = programs.iter().map(|p| p.id.as_str()).collect();
    for id in HO_BASE_IDS {
        assert!(ids.contains(&id), "missing {id}");
    }
    assert!(ids.contains(&"HO-HP01"));
    assert!(ids.contains(&"HO-HP22-QUIT"));
    assert!(ids.contains(&"HO-ROUTE-42"));
    assert!(ids.contains(&"HO-CLI-PREVIEW"));
    assert!(ids.contains(&"HO-SHELL"));
}

#[test]
fn named_branches_attach_to_parents() {
    let programs = expand_scenarios();
    let ho_base_01 = programs.iter().find(|p| p.id == "HO-BASE-01").unwrap();
    assert!(
        ho_base_01
            .branches
            .contains(&"clock-firstuse-frame-0-1-40".into())
    );
    assert!(
        ho_base_01
            .branches
            .contains(&"clock-cadence-not-delta".into())
    );
    let preview = programs
        .iter()
        .find(|p| p.id == "HO-PREVIEW-SELECT")
        .unwrap();
    for branch in [
        "preview-copy-find-live",
        "preview-selection-sweep",
        "preview-ctrl-c-exit",
        "preview-end-selection-noop",
    ] {
        assert!(preview.branches.contains(&branch.to_owned()), "{branch}");
    }
    let args = programs.iter().find(|p| p.id == "HO-ARGS").unwrap();
    for branch in [
        "args-insert-fallback",
        "args-valid-replacement",
        "args-required-empty",
        "args-nonnumeric-fallback",
    ] {
        assert!(args.branches.contains(&branch.to_owned()), "{branch}");
    }
    let overlay = programs
        .iter()
        .find(|p| p.id == "HO-OVERLAY-PASTE")
        .unwrap();
    assert_eq!(
        overlay
            .branches
            .iter()
            .filter(|b| b.starts_with("paste-"))
            .count(),
        8
    );
    assert_eq!(NAMED_BRANCHES.len(), 33);
}

#[test]
fn structured_verbs_keys_and_routes_expand() {
    let programs = expand_scenarios();
    assert!(
        programs
            .iter()
            .any(|p| p.actions.iter().any(|a| matches!(a, Action::Route { .. }))),
        "ROUTE seeds must expand"
    );
    assert!(
        programs
            .iter()
            .any(|p| p.actions.iter().any(|a| matches!(a, Action::Key(_)))),
        "bare key tokens must expand"
    );
    assert!(
        programs
            .iter()
            .any(|p| p.actions.iter().any(|a| matches!(a, Action::Sweep))),
        "SWEEP markers must expand"
    );
    assert!(
        programs
            .iter()
            .any(|p| p.actions.iter().any(|a| matches!(a, Action::Resize(_)))),
        "explicit resize lists must expand"
    );
}

#[test]
fn native_resize_map_covers_every_site() {
    let map = native_resize_map();
    assert_eq!(map.len(), 986);
    assert!(map.iter().all(|row| row.sizes.len() == 4));
    assert!(map.iter().all(|row| !row.site_id.is_empty()));
    assert!(map.iter().all(|row| !row.source_sha256.is_empty()));
    let status_click = map
        .iter()
        .find(|row| row.site_id == "src/bin/holla/app_tests_flows.rs:L745")
        .expect("mandatory (3,38) status click site");
    for size in &status_click.sizes {
        assert_eq!(
            size.pointer,
            Some((3, size.status_row)),
            "status click follows draw_frame bottom-2 at {}x{}",
            size.width,
            size.height
        );
    }
    assert!(
        map.iter()
            .filter(|row| row.source_line.contains("resize("))
            .all(|row| row.sizes.iter().all(|s| s.pointer.is_none())),
        "resize events are never pointer targets"
    );
}

#[test]
fn stage_membership_is_nonempty_and_complete() {
    assert_eq!(audit_row_count(), 135);
    let contrib = contributions();
    assert_eq!(contrib.len(), 23);
    assert!(contrib.iter().all(|row| row.minimum_checkpoint_count >= 1));
    assert!(shell_contribution_count() >= 1);
    assert!(shell_frame_contribution_count() >= 1);
}

#[test]
fn oracle_worlds_match_ho_base_fixture_order() {
    let programs = expand_scenarios();
    for (index, world) in ORACLE_WORLDS.iter().enumerate() {
        assert_eq!(programs[index].id, HO_BASE_IDS[index]);
        assert_eq!(programs[index].fixture, *world);
    }
}
