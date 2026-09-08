//! Source-bound fixture graph: pinned 794b095 domain/fixtures.rs302–448,1298–1680.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Source-bound fixture assertions fail directly when expected records disappear"
)]

use jackin_app::domain::instance::{DaemonSnapshot, InstanceStatus, RunId};
use jackin_app::{Scenario, world_for};

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "One explicit source ledger covers all durable fixture records"
)]
fn all_scenarios_preserve_source_durable_identity_and_history() {
    let expected = [
        (
            "jk-7f3a",
            Some(1),
            "chainargos/the-architect",
            InstanceStatus::Running,
            1_788_393_600,
            1_788_401_637,
            2,
            1,
            "/workspace/payments-platform",
        ),
        (
            "jk-c41e",
            Some(1),
            "chainargos/reviewer",
            InstanceStatus::PreservedDirty,
            1_788_304_440,
            1_788_315_240,
            3,
            0,
            "/workspace/payments-platform",
        ),
        (
            "jk-9b02",
            Some(2),
            "chainargos/sre",
            InstanceStatus::Running,
            1_788_399_240,
            1_788_401_638,
            0,
            0,
            "/workspace/infra-control-plane",
        ),
        (
            "jk-12ee",
            Some(4),
            "local/writer",
            InstanceStatus::Crashed,
            1_788_383_640,
            1_788_387_240,
            0,
            0,
            "/workspace/customer-portal",
        ),
        (
            "jk-a1c0",
            Some(3),
            "chainargos/backend",
            InstanceStatus::RestoreAvailable,
            1_788_142_440,
            1_788_142_440,
            0,
            0,
            "/workspace/release-automation",
        ),
        (
            "jk-04d7",
            Some(2),
            "chainargos/sre",
            InstanceStatus::PreservedUnpushed,
            1_788_228_840,
            1_788_228_840,
            0,
            2,
            "/workspace/infra-control-plane",
        ),
        (
            "jk-77aa",
            Some(1),
            "chainargos/backend",
            InstanceStatus::Superseded,
            1_787_883_240,
            1_787_883_240,
            0,
            0,
            "/workspace/payments-platform",
        ),
        (
            "jk-88bb",
            Some(4),
            "local/writer",
            InstanceStatus::Purged,
            1_787_624_040,
            1_787_624_040,
            0,
            0,
            "/workspace/customer-portal",
        ),
        (
            "jk-5e5e",
            Some(3),
            "chainargos/backend",
            InstanceStatus::FailedSetup,
            1_788_399_840,
            1_788_399_840,
            0,
            0,
            "/workspace/release-automation",
        ),
        (
            "jk-e0e0",
            Some(10),
            "chainargos/backend",
            InstanceStatus::Running,
            1_788_401_040,
            1_788_401_550,
            0,
            0,
            "/workspace/data-pipeline",
        ),
        (
            "jk-f1f1",
            Some(11),
            "local/writer",
            InstanceStatus::CleanExited,
            1_788_056_040,
            1_788_056_040,
            0,
            0,
            "/workspace/docs-site",
        ),
        (
            "jk-b000",
            Some(1),
            "chainargos/reviewer",
            InstanceStatus::CleanExited,
            1_788_228_840,
            1_788_228_840,
            0,
            0,
            "/workspace/payments-platform",
        ),
        (
            "jk-b111",
            Some(1),
            "chainargos/reviewer",
            InstanceStatus::RestoreAvailable,
            1_788_142_440,
            1_788_142_440,
            0,
            0,
            "/workspace/payments-platform",
        ),
        (
            "jk-b222",
            Some(1),
            "chainargos/reviewer",
            InstanceStatus::CleanExited,
            1_788_056_040,
            1_788_056_040,
            0,
            0,
            "/workspace/payments-platform",
        ),
        (
            "jk-b333",
            Some(1),
            "chainargos/reviewer",
            InstanceStatus::RestoreAvailable,
            1_787_969_640,
            1_787_969_640,
            0,
            0,
            "/workspace/payments-platform",
        ),
        (
            "jk-b444",
            Some(1),
            "chainargos/reviewer",
            InstanceStatus::CleanExited,
            1_787_883_240,
            1_787_883_240,
            0,
            0,
            "/workspace/payments-platform",
        ),
        (
            "jk-b555",
            Some(1),
            "chainargos/reviewer",
            InstanceStatus::RestoreAvailable,
            1_787_796_840,
            1_787_796_840,
            0,
            0,
            "/workspace/payments-platform",
        ),
    ];
    for scenario in Scenario::ALL {
        let w = world_for(scenario);
        assert_eq!(
            w.roles,
            jackin_app::domain::fixtures::fixture_roles_for(scenario)
        );
        assert_eq!(
            w.workspaces,
            jackin_app::domain::fixtures::fixture_workspaces_for(scenario)
        );
        let len = if scenario == Scenario::FirstUse {
            0
        } else if scenario == Scenario::HardCases {
            17
        } else {
            9
        };
        assert_eq!(w.instances.len(), len, "{scenario:?}");
        for (actual, &(id, owner, role, mut status, created, seen, dirty, unpushed, workdir)) in
            w.instances.iter().zip(&expected[..len])
        {
            if scenario == Scenario::OutroLast && id == "jk-9b02" {
                status = InstanceStatus::CleanExited;
            }
            assert_eq!(
                (
                    actual.id.as_str(),
                    actual.workspace,
                    actual.role.as_str(),
                    actual.status,
                    actual.created_secs,
                    actual.last_seen_secs,
                    actual.uncommitted,
                    actual.unpushed,
                    actual.workdir.as_str()
                ),
                (
                    id, owner, role, status, created, seen, dirty, unpushed, workdir
                ),
                "{scenario:?}"
            );
            assert_eq!(
                actual.run_id,
                RunId::from_label(&format!("run-{}", &id[3..]))
            );
            assert!(w.roles.iter().any(|r| r.full_name() == actual.role));
            assert!(
                actual
                    .accounts
                    .iter()
                    .all(|id| w.accounts.get(id).is_some())
            );
        }
        let running = match scenario {
            Scenario::FirstUse => 0,
            Scenario::HardCases => 3,
            Scenario::OutroLast => 1,
            _ => 2,
        };
        assert_eq!(w.running_count(), running);
        assert_eq!(
            w.roles.len(),
            match scenario {
                Scenario::FirstUse => 6,
                Scenario::HardCases => 126,
                _ => 46,
            }
        );
        assert_eq!(
            w.workspaces.len(),
            match scenario {
                Scenario::FirstUse => 0,
                Scenario::HardCases => 14,
                _ => 4,
            }
        );
    }
}

#[test]
fn source_registry_has_one_canonical_account_set_and_coherent_workspace_policies() {
    let w = world_for(Scenario::Returning);
    let ws = w.workspace(1).unwrap();
    assert_eq!(ws.workdir, "/workspace/payments-platform");
    assert_eq!(
        ws.roles.default.as_deref(),
        Some("chainargos/the-architect")
    );
    assert_eq!(ws.roles.last.as_deref(), Some("chainargos/the-architect"));
    assert_eq!(w.workspace(3).unwrap().name, "release-automation");
    assert_eq!(w.workspace(4).unwrap().name, "customer-portal");
    assert_eq!(
        w.accounts
            .default_for(jackin_app::domain::agent::Provider::Anthropic)
            .unwrap()
            .id,
        "acct-claude-personal"
    );
    assert!(w.accounts.get("anthropic-work").is_none());
    assert!(w.accounts.get("openai-primary").is_none());
    assert_eq!(w.cwd_workspace().map(|w| w.id), Some(1));
}

#[test]
fn source_manifest_and_daemon_are_independent_and_outro_keeps_history() {
    let w = world_for(Scenario::CapsuleMulti);
    let primary = w.instance("jk-7f3a").unwrap();
    assert_eq!(
        primary.branch.as_deref(),
        Some("feature/settlement-backoff")
    );
    assert_eq!(
        primary.pr.as_ref().map(|p| (p.0, p.1.as_str())),
        Some((482, "Settlement retry backoff"))
    );
    assert_eq!(primary.sessions.as_ref().unwrap().len(), 2);
    let daemon = &w.daemons["jk-7f3a"];
    assert_eq!(daemon.tabs.len(), 3);
    assert_eq!(daemon.panes.len(), 5);
    assert_eq!(daemon.active, 0);
    assert_eq!(daemon.tabs[0].focused, 1);
    assert_eq!(daemon.tabs[2].custom_label.as_deref(), Some("docs"));
    assert_eq!(w.daemons["jk-9b02"].panes.len(), 1);
    let hard = world_for(Scenario::HardCases);
    assert!(hard.instance("jk-e0e0").unwrap().sessions.is_err());
    assert_eq!(
        hard.instance("jk-e0e0").unwrap().daemon,
        DaemonSnapshot::Unavailable
    );
    assert!(!hard.daemons.contains_key("jk-e0e0"));
    let outro = world_for(Scenario::OutroLast);
    assert_eq!(
        outro.instance("jk-9b02").unwrap().status,
        InstanceStatus::CleanExited
    );
    assert!(!outro.daemons.contains_key("jk-9b02"));
    assert_eq!(outro.arbiter.entered_at_ms, Some(-8_040_000));
}

#[test]
fn effective_accounts_and_role_references_remain_closed_over_the_catalog() {
    use jackin_app::domain::workspace::AllowedRoles;
    for scenario in Scenario::ALL {
        let world = world_for(scenario);
        assert_eq!(
            world.accounts.accounts.len(),
            match scenario {
                Scenario::FirstUse => 0,
                Scenario::HardCases => 13,
                _ => 12,
            }
        );
        for ws in &world.workspaces {
            let known = |role: &str| world.roles.iter().any(|entry| entry.full_name() == role);
            if let AllowedRoles::Custom(roles) = &ws.roles.allowed {
                assert!(roles.iter().all(|role| known(role)));
            }
            assert!(
                ws.roles
                    .default
                    .iter()
                    .chain(ws.roles.last.iter())
                    .all(|role| known(role))
            );
            assert!(ws.role_env.keys().all(|role| known(role)));
            assert!(
                ws.accounts.role_preferred.iter().all(
                    |((role, _), account)| known(role) && world.accounts.get(account).is_some()
                )
            );
            assert!(
                ws.accounts
                    .enabled
                    .iter()
                    .chain(ws.accounts.disabled_defaults.iter())
                    .chain(ws.accounts.preferred.values())
                    .all(|id| world.accounts.get(id).is_some())
            );
            let effective: Vec<_> = ws
                .effective_accounts(&world.accounts)
                .into_iter()
                .map(|account| account.id)
                .collect();
            for instance in world
                .instances
                .iter()
                .filter(|instance| instance.workspace == Some(ws.id))
            {
                assert_eq!(instance.accounts, effective);
            }
        }
    }
}

#[test]
fn public_launch_routes_preserve_preexisting_construct_members() {
    use jackin_app::{App, Motion, Route};
    use junie_tui::Theme;
    use junie_tui_testing::Harness;
    for scenario in [Scenario::LaunchRunning, Scenario::LaunchFailure] {
        let mut h = Harness::new(
            App::for_scenario(scenario, Motion::Reduced),
            Theme::junie(),
            120,
            40,
        );
        assert_eq!(h.app().route(), Route::Cockpit);
        assert_eq!(h.app().world.running_count(), 2);
        for _ in 0..600 {
            let _ = h.advance(std::time::Duration::from_millis(200));
            if h.app().route() == Route::Capsule || h.text().contains("Launch failed") {
                break;
            }
        }
        if scenario == Scenario::LaunchRunning {
            assert_eq!(h.app().route(), Route::Capsule);
        } else {
            assert!(h.text().contains("Launch failed"));
        }
        let world = &h.app().world;
        assert_eq!(
            world.instance("jk-7f3a").unwrap().status,
            InstanceStatus::Running
        );
        assert_eq!(
            world.instance("jk-9b02").unwrap().status,
            InstanceStatus::Running
        );
        assert_eq!(
            world.running_count(),
            if scenario == Scenario::LaunchRunning {
                3
            } else {
                2
            }
        );
    }
}

#[test]
fn actual_initial_routes_draw_without_mutating_restored_domain_records() {
    use jackin_app::{App, Motion};
    use junie_tui::Theme;
    use junie_tui_testing::Harness;
    for scenario in Scenario::ALL {
        let mut h = Harness::new(
            App::for_scenario(scenario, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let instances = h.app().world.instances.clone();
        let workspaces = h.app().world.workspaces.clone();
        let accounts = h.app().world.accounts.clone();
        let time = h.app().world.now_ms();
        let route = h.app().route();
        for _ in 0..3 {
            h.draw();
        }
        assert_eq!(h.app().route(), route);
        assert_eq!(h.app().world.instances, instances);
        assert_eq!(h.app().world.workspaces, workspaces);
        assert_eq!(h.app().world.accounts, accounts);
        assert_eq!(h.app().world.now_ms(), time);
        assert!(!h.text().trim().is_empty());
    }
}

#[test]
fn adding_role_environment_retains_the_chosen_catalog_key() {
    use jackin_app::{App, Motion, Route};
    use junie_tui::{Id, KeyCode, Theme};
    use junie_tui_testing::Harness;
    let mut h = Harness::new(
        App::for_scenario(Scenario::HardCases, Motion::Reduced),
        Theme::junie(),
        120,
        40,
    );
    for _ in 0..8 {
        for _ in 0..3 {
            let _ = h.advance(std::time::Duration::from_millis(200));
        }
        if h.app().route() == Route::Manager {
            break;
        }
        let _ = h.key(KeyCode::Enter);
    }
    for key in [
        KeyCode::Down,
        KeyCode::Char('e'),
        KeyCode::Char('4'),
        KeyCode::Enter,
        KeyCode::End,
        KeyCode::Enter,
    ] {
        let _ = h.key(key);
    }
    assert!(h.text().contains("Add role override"));
    let _ = h.type_str("svc-01");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("New svc-010 environment key"));
    let _ = h.key(KeyCode::Enter);
    let _ = h.type_str("SVC_FLAG");
    for key in [KeyCode::Tab, KeyCode::Tab, KeyCode::Enter] {
        let _ = h.key(key);
    }
    let _ = h.type_str("on");
    let _ = h.key(KeyCode::Tab);
    assert!(h.tab_to(Id::root("editor.cfg").sub("form").sub("save")));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.app()
            .editor
            .pending
            .role_env
            .contains_key("chainargos/svc-010")
    );
    assert!(!h.app().editor.pending.role_env.contains_key("svc-010"));
    assert!(h.app().editor.pending.role_env.keys().all(|key| {
        h.app()
            .world
            .roles
            .iter()
            .any(|role| role.full_name() == *key)
    }));
}
