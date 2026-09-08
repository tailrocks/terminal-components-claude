//! One invocation boundary for keys, pointer activation and modal actions.
use crate::domain::{
    action::{Action, ActionKind, Availability, Risk},
    activity::{Activity, ActivityState},
    docker::ContainerState,
};
use crate::scenario::Scenario;
use crate::sim::{
    catalogue::{self, IntentError},
    plans::{self, ReviewedPlan},
    world::World,
};

pub(crate) enum Destination {
    Notice(String),
    Preview(Action),
    Trust { action: Action, file: String },
    Plan(Box<ReviewedPlan>),
    Database,
    Monitor,
    Clone,
    Activity(u32, String),
}

/// Canonical dispatch always re-resolves the selected row. Memory-row identities
/// remain useful for selection but never select the simulated effect handler.
pub(crate) fn invoke(world: &mut World, row: &Action) -> Destination {
    let action = match catalogue::resolve_intent(world, row) {
        Ok(action) => action,
        Err(IntentError::NeedsTrust) => {
            let Some(current) = catalogue::catalogue(world)
                .into_iter()
                .find(|action| action.id == row.id)
            else {
                return Destination::Notice(IntentError::StaleTarget.to_string());
            };
            let Availability::NeedsTrust(file) = &current.availability else {
                return Destination::Notice(IntentError::StaleTarget.to_string());
            };
            return Destination::Trust {
                file: file.clone(),
                action: current,
            };
        }
        Err(error) => return Destination::Notice(error.to_string()),
    };
    match action.intent_id() {
        Some("pg.locks") => return Destination::Database,
        Some("flow.monitor") => return Destination::Monitor,
        Some("gh.clone") => return Destination::Clone,
        _ => {}
    }
    match action.risk {
        Risk::Broad => plans::plan_for(world, action.intent_id().unwrap_or_default()).map_or_else(
            || Destination::Notice(format!("{} · no plan available", action.title)),
            |review| Destination::Plan(Box::new(review)),
        ),
        Risk::Bounded => Destination::Preview(action),
        Risk::ReadOnly => simulate(world, &action),
    }
}

/// The preview's confirmation returns through the same current-policy lookup.
/// A policy that became broad must enter the compound review, never run here.
pub(crate) fn confirm_preview(world: &mut World, row: &Action) -> Destination {
    match catalogue::resolve_intent(world, row) {
        Ok(action) if action.risk != Risk::Broad => simulate(world, &action),
        _ => invoke(world, row),
    }
}

fn next_activity_id(world: &World) -> Option<u32> {
    world
        .activities
        .iter()
        .map(|activity| activity.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
}
fn simulate(world: &mut World, action: &Action) -> Destination {
    if action.long_running {
        return start_activity(world, action);
    }
    if world.scenario == Scenario::LaunchFailure && action.kind == ActionKind::Task {
        let Some(id) = next_activity_id(world) else {
            return Destination::Notice("Activity identity exhausted · nothing started".into());
        };
        let mut activity = Activity::new(
            id,
            &action.title,
            workdir_label(action),
            ActivityState::Failed,
            world.now_ms(),
        );
        activity.lines = vec![
            format!("$ {}", action.command),
            "test cache::stores ... FAILED".into(),
            "error: assertion failed · exit code 1".into(),
        ];
        world.activities.push(activity);
        return Destination::Notice(format!("{} failed · exit code 1 · simulated", action.title));
    }
    Destination::Notice(format!(
        "Would run: {} · simulated, nothing executed",
        action.command
    ))
}
fn start_activity(world: &mut World, action: &Action) -> Destination {
    let Some(id) = next_activity_id(world) else {
        return Destination::Notice("Activity identity exhausted · nothing started".into());
    };
    if action.intent_id() == Some("docker.logs") {
        let lines = merged_log_lines(world);
        if lines.is_empty() {
            return Destination::Notice("No running containers to follow".into());
        }
        let mut activity = Activity::new(
            id,
            "container logs",
            &world.host.name,
            ActivityState::Running,
            world.now_ms(),
        );
        activity.lines = lines;
        world.activities.push(activity);
        return Destination::Activity(id, "Following container logs · merged stream".into());
    }
    let mut activity = Activity::new(
        id,
        &action.title,
        workdir_label(action),
        ActivityState::Running,
        world.now_ms(),
    );
    activity.lines = vec![
        format!("$ {}", action.command),
        "started · simulated".into(),
    ];
    world.activities.push(activity);
    Destination::Notice(format!("Activity started: {} · simulated", action.title))
}
fn workdir_label(action: &Action) -> &str {
    if action.workdir.is_empty() {
        &action.target
    } else {
        &action.workdir
    }
}
fn merged_log_lines(world: &World) -> Vec<String> {
    let Some(docker) = &world.docker else {
        return Vec::new();
    };
    let mut names: Vec<_> = docker
        .containers
        .iter()
        .filter(|container| container.state == ContainerState::Running)
        .map(|container| container.name.as_str())
        .collect();
    names.sort_unstable();
    let samples: Vec<_> = names.iter().map(|name| log_sample(name)).collect();
    let depth = samples.iter().map(|lines| lines.len()).max().unwrap_or(0);
    let mut output = Vec::new();
    for round in 0..depth {
        for (name, lines) in names.iter().zip(&samples) {
            if let Some(line) = lines.get(round) {
                output.push(format!("{name:<8} | {line}"));
            }
        }
    }
    output
}
fn log_sample(name: &str) -> &'static [&'static str] {
    match name {
        "api" => &[
            "10:24:01 GET /health 200 · 2ms",
            "10:24:03 POST /orders 201 · 41ms",
            "10:24:09 GET /health 200 · 1ms",
            "10:24:12 GET /orders/9918 200 · 6ms",
        ],
        "worker" => &[
            "10:24:02 job 1148 done · 310ms",
            "10:24:06 job 1149 done · 288ms",
            "10:24:10 job 1150 running",
        ],
        "redis" => &[
            "10:24:01 3 clients connected",
            "10:24:05 background save done",
            "10:24:11 keyspace hits 10422 misses 91",
        ],
        "postgres" => &[
            "10:24:02 checkpoint complete",
            "10:24:08 autovacuum payments",
        ],
        "nginx" => &["10:24:03 200 GET /", "10:24:07 304 GET /assets/app.css"],
        "gitea" => &["10:24:04 GET /api/v1/version 200"],
        _ => &["10:24:00 alive", "10:24:06 alive"],
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "Named deterministic actions must exist in the fixture"
)]
#[expect(
    clippy::panic,
    reason = "Fixed test fixtures must reach their typed route"
)]
mod tests {
    use super::*;
    use crate::domain::{fixtures, ranking::Pin};
    fn settled(scenario: Scenario) -> World {
        let mut world = fixtures::world_for(scenario);
        world.seek(4_000);
        world
    }
    fn action(world: &World, id: &str) -> Action {
        catalogue::catalogue(world)
            .into_iter()
            .find(|action| action.id == id)
            .unwrap()
    }
    #[test]
    fn clone_uses_the_same_flow_for_canonical_and_remembered_rows() {
        let mut world = settled(Scenario::RustDirty);
        let original = action(&world, "gh.clone");
        assert!(matches!(invoke(&mut world, &original), Destination::Clone));
        world.memory.pins.push(Pin {
            path: world.cwd.clone(),
            command: original.command.clone(),
        });
        let remembered = catalogue::catalogue(&world)
            .into_iter()
            .find(|row| row.id.starts_with("pin:") && row.intent_id() == Some("gh.clone"))
            .unwrap();
        assert!(matches!(
            invoke(&mut world, &remembered),
            Destination::Clone
        ));
    }
    #[test]
    fn broad_memory_never_bypasses_review_and_stale_target_never_runs() {
        let mut world = settled(Scenario::DockerCleanup);
        let original = action(&world, "docker.cleanup");
        world.memory.pins.push(Pin {
            path: world.cwd.clone(),
            command: original.command.clone(),
        });
        let remembered = catalogue::catalogue(&world)
            .into_iter()
            .find(|row| row.id.starts_with("pin:") && row.intent_id() == Some("docker.cleanup"))
            .unwrap();
        assert!(matches!(
            invoke(&mut world, &remembered),
            Destination::Plan(_)
        ));
        assert!(matches!(
            confirm_preview(&mut world, &remembered),
            Destination::Plan(_)
        ));
        assert_eq!(world.effect_revision, 0);
        let mut stale = original;
        stale.target = "different-target".into();
        assert!(matches!(invoke(&mut world, &stale), Destination::Notice(_)));
        assert_eq!(world.effect_revision, 0);
    }
    #[test]
    fn merged_logs_keep_sorted_service_identity_and_navigate_to_activity() {
        let mut world = settled(Scenario::DockerCleanup);
        let logs = action(&world, "docker.logs");
        let Destination::Activity(id, message) = invoke(&mut world, &logs) else {
            panic!("logs must become an activity");
        };
        assert_eq!(message, "Following container logs · merged stream");
        let output = &world
            .activities
            .iter()
            .find(|activity| activity.id == id)
            .unwrap()
            .lines;
        assert!(output.iter().all(|line| line.contains(" | ")));
        assert!(
            output
                .iter()
                .any(|line| line == "api      | 10:24:01 GET /health 200 · 2ms")
        );
        assert!(
            output
                .iter()
                .any(|line| line == "worker   | 10:24:10 job 1150 running")
        );
    }
    #[test]
    fn activity_id_exhaustion_refuses_without_overflow_or_duplicate_identity() {
        let mut world = settled(Scenario::DockerCleanup);
        world.activities.push(Activity::new(
            u32::MAX,
            "last",
            "fixture",
            ActivityState::Running,
            0,
        ));
        let logs = action(&world, "docker.logs");
        let before = world.activities.clone();
        assert!(matches!(invoke(&mut world, &logs), Destination::Notice(_)));
        assert_eq!(world.activities, before);
    }
}
