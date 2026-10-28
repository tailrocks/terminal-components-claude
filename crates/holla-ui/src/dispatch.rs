use crate::domain::{
    action::{Action, ActionKind, Availability, Risk},
    activity::{Activity, ActivityState},
    docker::ContainerState,
};
use crate::scenario::Scenario;
use crate::sim::activity_supervisor::{
    ActivityExecutionId, ActivityExecutionStart, ActivityStopProfile,
};
use crate::sim::{
    catalogue::{self, IntentError},
    plans::{self, ReviewedPlan},
    port_args::{
        PortArgsProducer, PortArgsSession, PortInvocationPlan, PortReviewFacts,
        port_args_producer_for, port_review_facts, simulate_port_invocation,
    },
    task_args::{
        InvocationPlan, SafeReviewFacts, TaskArgsProducer, TaskArgsSession, nextest_review_facts,
        simulate_nextest_invocation, task_args_producer_for,
    },
    world::World,
};

pub(crate) enum Destination {
    Notice(String),
    Preview(Action),
    Trust {
        action: Action,
        file: String,
        args: Option<Box<TaskArgsStart>>,
    },
    PortArgs(Box<PortArgsStart>),
    PortArgsReview {
        producer: PortArgsProducer,
        plan: Box<PortInvocationPlan>,
        facts: PortReviewFacts,
    },
    TaskArgs(Box<TaskArgsStart>),
    TaskArgsReview {
        producer: TaskArgsProducer,
        plan: Box<InvocationPlan>,
        facts: SafeReviewFacts,
    },
    TaskRunStart {
        action: Action,
        script: crate::sim::task_run::TaskScriptId,
    },
    Plan(Box<ReviewedPlan>),
    Database,
    Monitor,
    Clone,
    Snapshot(crate::screens::snapshot::Kind),
    DiskScan,
    HostScope,
    Activity {
        id: u32,
        message: String,
        execution_id: ActivityExecutionId,
    },
    ActivityNotice {
        id: u32,
        message: String,
        execution_id: ActivityExecutionId,
    },
}

pub(crate) struct PortArgsStart {
    pub(crate) action: Action,
    pub(crate) producer: PortArgsProducer,
    pub(crate) session: PortArgsSession,
}

pub(crate) struct TaskArgsStart {
    pub(crate) action: Action,
    pub(crate) producer: TaskArgsProducer,
    pub(crate) session: TaskArgsSession,
}

pub(crate) fn preview(world: &mut World, row: &Action) -> Destination {
    match port_args_producer_for(row) {
        Ok(Some(producer)) => return start_port_args(world, row, producer),
        Err(error) => return Destination::Notice(error.to_string()),
        Ok(None) => {}
    }
    match task_args_producer_for(row) {
        Ok(Some(producer)) => return start_task_args(world, row, producer),
        Err(error) => return Destination::Notice(error.to_string()),
        Ok(None) => {}
    }
    if let Some(destination) = task_run_destination(world, row, false) {
        return destination;
    }
    match &row.availability {
        Availability::NeedsTrust(file) => Destination::Trust {
            action: row.clone(),
            file: file.clone(),
            args: None,
        },
        _ => Destination::Preview(row.clone()),
    }
}

pub(crate) fn invoke(world: &mut World, row: &Action) -> Destination {
    match port_args_producer_for(row) {
        Ok(Some(producer)) => return start_port_args(world, row, producer),
        Err(error) => return Destination::Notice(error.to_string()),
        Ok(None) => {}
    }
    let producer = match task_args_producer_for(row) {
        Ok(producer) => producer,
        Err(error) => return Destination::Notice(error.to_string()),
    };
    if let Some(producer) = producer {
        return start_task_args(world, row, producer);
    }
    if let Some(destination) = task_run_destination(world, row, true) {
        return destination;
    }

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
            let file = file.clone();
            return Destination::Trust {
                file,
                action: current,
                args: None,
            };
        }
        Err(error) => return Destination::Notice(error.to_string()),
    };
    match action.intent_id() {
        Some("system.resources") => {
            return Destination::Snapshot(crate::screens::snapshot::Kind::Resources);
        }
        Some("system.processes") => {
            return Destination::Snapshot(crate::screens::snapshot::Kind::Processes);
        }
        Some("disk.usage") => return Destination::DiskScan,
        Some("scope.system") => return Destination::HostScope,
        Some("system.monitor") => {}
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

const DEV_TASK_ID: &str = "dev";
const DEV_TASK_FILE: &str = "~/work/monorepo/apps/frontend/mise.toml";
const DEV_TASK_BODY: &str = "vite --port 5199";
const DEV_TASK_WORKDIR: &str = "~/work/monorepo/apps/frontend";
const DEV_TASK_COMMAND: &str = "mise run dev";

fn registered_dev_action(action: &Action) -> bool {
    action.id == "task:dev"
        && action.intent_id() == Some("task:dev")
        && action.kind == ActionKind::Task
        && action.scope == crate::domain::action::Scope::Here
        && action.risk == Risk::ReadOnly
        && action.target == DEV_TASK_WORKDIR
        && action.workdir == DEV_TASK_WORKDIR
        && action.command == DEV_TASK_COMMAND
        && action.keywords == DEV_TASK_BODY
}

fn same_task_run_action(selected: &Action, current: &Action) -> bool {
    selected.id == current.id
        && selected.intent() == current.intent()
        && selected.kind == current.kind
        && selected.scope == current.scope
        && selected.risk == current.risk
        && selected.target == current.target
        && selected.command == current.command
        && selected.workdir == current.workdir
        && selected.keywords == current.keywords
}

fn task_run_destination(world: &World, selected: &Action, launch: bool) -> Option<Destination> {
    if selected.id != "task:dev" {
        return None;
    }

    let selected_is_registered = registered_dev_action(selected);
    let dev_tasks: Vec<_> = world
        .mise
        .as_ref()
        .into_iter()
        .flat_map(|mise| mise.tasks.iter())
        .filter(|task| task.id == DEV_TASK_ID)
        .collect();
    let current_is_registered = dev_tasks
        .iter()
        .any(|task| task.defined_in == DEV_TASK_FILE && task.command == DEV_TASK_BODY);
    if !selected_is_registered && !current_is_registered {
        return None;
    }
    if dev_tasks.len() != 1 {
        return Some(Destination::Notice(
            "Task source changed · review it again".into(),
        ));
    }
    let Some(task) = dev_tasks.first().copied() else {
        return Some(Destination::Notice(
            "Task source changed · review it again".into(),
        ));
    };
    if task.defined_in != DEV_TASK_FILE || task.command != DEV_TASK_BODY {
        return Some(Destination::Notice(
            "Task source changed · review it again".into(),
        ));
    }

    let mut actions = catalogue::catalogue(world)
        .into_iter()
        .filter(|action| action.id == "task:dev");
    let Some(current) = actions.next() else {
        return Some(Destination::Notice(
            "Task source changed · review it again".into(),
        ));
    };
    if actions.next().is_some()
        || !registered_dev_action(&current)
        || !same_task_run_action(selected, &current)
    {
        return Some(Destination::Notice(
            "Task source changed · review it again".into(),
        ));
    }

    match catalogue::resolve_intent(world, selected) {
        Ok(action)
            if registered_dev_action(&action)
                && same_task_run_action(selected, &action)
                && matches!(&action.availability, Availability::Ready) =>
        {
            if launch {
                Some(Destination::TaskRunStart {
                    action,
                    script: crate::sim::task_run::TaskScriptId::DevServerDemo,
                })
            } else {
                Some(Destination::Preview(action))
            }
        }
        Err(IntentError::NeedsTrust) => {
            let Availability::NeedsTrust(file) = &current.availability else {
                return Some(Destination::Notice(
                    "Task source changed · review it again".into(),
                ));
            };
            let file = file.clone();
            Some(Destination::Trust {
                action: current,
                file,
                args: None,
            })
        }
        _ => Some(Destination::Notice(
            "Task source changed · review it again".into(),
        )),
    }
}

pub(crate) fn task_run_source_is_current(world: &World, action: &Action) -> bool {
    matches!(
        task_run_destination(world, action, true),
        Some(Destination::TaskRunStart { .. })
    )
}

fn start_port_args(world: &World, row: &Action, producer: PortArgsProducer) -> Destination {
    let session = match PortArgsSession::begin(world, row, &producer) {
        Ok(session) => session,
        Err(error) => return Destination::Notice(error.to_string()),
    };
    match &row.availability {
        Availability::Ready => Destination::PortArgs(Box::new(PortArgsStart {
            action: row.clone(),
            producer,
            session,
        })),
        Availability::NeedsTrust(_) | Availability::Blocked(_) => {
            Destination::Notice("port action is not a ready read-only host action".into())
        }
    }
}

fn start_task_args(world: &World, row: &Action, producer: TaskArgsProducer) -> Destination {
    let session = match TaskArgsSession::begin(world, row, &producer) {
        Ok(session) => session,
        Err(error) => return Destination::Notice(error.to_string()),
    };
    match &row.availability {
        Availability::NeedsTrust(file) => Destination::Trust {
            action: row.clone(),
            file: file.clone(),
            args: Some(Box::new(TaskArgsStart {
                action: row.clone(),
                producer,
                session,
            })),
        },
        Availability::Ready => Destination::TaskArgs(Box::new(TaskArgsStart {
            action: row.clone(),
            producer,
            session,
        })),
        Availability::Blocked(reason) => Destination::Notice(reason.clone()),
    }
}

pub(crate) fn review_port_args(
    world: &World,
    producer: PortArgsProducer,
    plan: PortInvocationPlan,
) -> Destination {
    match port_review_facts(world, &producer, &plan) {
        Ok(facts) => Destination::PortArgsReview {
            producer,
            plan: Box::new(plan),
            facts,
        },
        Err(error) => Destination::Notice(error.to_string()),
    }
}

pub(crate) fn confirm_port_args(
    world: &World,
    producer: &PortArgsProducer,
    plan: PortInvocationPlan,
) -> Destination {
    match simulate_port_invocation(world, producer, &plan) {
        Ok(report) => Destination::Notice(format!(
            "Simulation confirmed · nothing executed · {}",
            crate::screens::args::format_invocation(&report.invocation)
        )),
        Err(error) => Destination::Notice(error.to_string()),
    }
}

pub(crate) fn review_task_args(
    world: &World,
    producer: TaskArgsProducer,
    plan: InvocationPlan,
) -> Destination {
    match nextest_review_facts(world, &producer, &plan) {
        Ok(facts) => Destination::TaskArgsReview {
            producer,
            plan: Box::new(plan),
            facts,
        },
        Err(error) => Destination::Notice(error.to_string()),
    }
}

pub(crate) fn confirm_task_args(
    world: &World,
    producer: &TaskArgsProducer,
    plan: InvocationPlan,
) -> Destination {
    match simulate_nextest_invocation(world, producer, &plan) {
        Ok(report) => Destination::Notice(format!(
            "Simulation confirmed · nothing executed · {}",
            crate::screens::args::format_invocation(&report.invocation)
        )),
        Err(error) => Destination::Notice(error.to_string()),
    }
}

pub(crate) fn confirm_preview(world: &mut World, row: &Action) -> Destination {
    match port_args_producer_for(row) {
        Ok(Some(producer)) => return start_port_args(world, row, producer),
        Err(error) => return Destination::Notice(error.to_string()),
        Ok(None) => {}
    }
    match task_args_producer_for(row) {
        Ok(Some(producer)) => return start_task_args(world, row, producer),
        Err(error) => return Destination::Notice(error.to_string()),
        Ok(None) => {}
    }
    if let Some(destination) = task_run_destination(world, row, true) {
        return destination;
    }
    match catalogue::resolve_intent(world, row) {
        Ok(action) if action.risk != Risk::Broad => simulate(world, &action),
        _ => invoke(world, row),
    }
}

pub(crate) fn next_activity_id(world: &World) -> Option<u32> {
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
        let execution_id = match world
            .activity_supervisor
            .register_terminal(&activity, ActivityStopProfile::Responsive)
        {
            Ok(id) => id,
            Err(error) => return Destination::Notice(format!("Activity unavailable · {error}")),
        };
        world.activities.push(activity);
        return Destination::ActivityNotice {
            id,
            message: format!("{} failed · exit code 1 · simulated", action.title),
            execution_id,
        };
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
        let execution_id = match world.activity_supervisor.register(
            &activity,
            ActivityExecutionStart::Detached,
            ActivityStopProfile::Responsive,
        ) {
            Ok(id) => id,
            Err(error) => return Destination::Notice(format!("Activity unavailable · {error}")),
        };
        world.activities.push(activity);
        return Destination::Activity {
            id,
            message: "Following container logs · merged stream".into(),
            execution_id,
        };
    }
    let activity = task_run_activity(world, action, id);
    let execution_id = match world.activity_supervisor.register(
        &activity,
        ActivityExecutionStart::Running,
        ActivityStopProfile::Responsive,
    ) {
        Ok(execution_id) => execution_id,
        Err(error) => return Destination::Notice(format!("Activity unavailable · {error}")),
    };
    world.activities.push(activity);
    Destination::ActivityNotice {
        id,
        message: format!("Activity started: {} · simulated", action.title),
        execution_id,
    }
}
fn workdir_label(action: &Action) -> &str {
    if action.workdir.is_empty() {
        &action.target
    } else {
        &action.workdir
    }
}

pub(crate) fn task_run_activity(world: &World, action: &Action, id: u32) -> Activity {
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
    activity
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
