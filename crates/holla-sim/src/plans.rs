use crate::domain::disk::{Candidate, DiskState, Freshness};
use crate::domain::docker::{ContainerState, Health};
use crate::domain::effect::Mutation;
use crate::domain::human_bytes;
use crate::domain::mise::ToolState;
use crate::domain::plan::{Plan, PlanEffect, PlanSpec, PlanStep, StepState};
use crate::world::World;
use std::sync::Arc;

pub struct ReviewedPlan {
    plan: Plan,
    target: Arc<TargetSnapshot>,
}

impl ReviewedPlan {
    fn new(plan: Plan, world: &World) -> Self {
        let target = Arc::new(TargetSnapshot::capture(&plan, world));
        Self { plan, target }
    }

    pub fn plan(&self) -> &Plan {
        &self.plan
    }

    pub fn binding(&self) -> ReviewBinding {
        ReviewBinding {
            target: Arc::clone(&self.target),
            plan: self.plan.clone(),
        }
    }

    pub fn matches_binding(&self, binding: &ReviewBinding) -> bool {
        Arc::ptr_eq(&self.target, &binding.target) && self.plan == binding.plan
    }

    pub fn toggle(&mut self, index: usize) -> Result<String, crate::domain::plan::PlanError> {
        self.plan.toggle(index)
    }

    pub fn approve<'a>(
        &'a mut self,
        phrase: &str,
        world: &World,
    ) -> Result<Approval<'a>, ApprovalError> {
        self.validate(world)?;
        if phrase != self.plan.phrase() {
            return Err(ApprovalError::WrongPhrase);
        }
        Ok(Approval { review: self })
    }

    fn validate(&self, world: &World) -> Result<(), ApprovalError> {
        if self.plan.ran() {
            return Err(ApprovalError::AlreadyConsumed);
        }
        if self.target.as_ref() != &TargetSnapshot::capture(&self.plan, world) {
            return Err(ApprovalError::StaleTarget);
        }
        Ok(())
    }
}

pub struct ReviewBinding {
    target: Arc<TargetSnapshot>,
    plan: Plan,
}

pub struct Approval<'a> {
    review: &'a mut ReviewedPlan,
}

impl Approval<'_> {
    pub fn execute(self, world: &mut World) -> Result<EffectReport, ApprovalError> {
        self.review.validate(world)?;
        let next_revision = world
            .effect_revision
            .checked_add(1)
            .ok_or(ApprovalError::RevisionExhausted)?;

        let mut completed = self.review.plan.clone();
        completed
            .run()
            .map_err(|_| ApprovalError::AlreadyConsumed)?;
        let steps = completed.steps();
        let mut applied_steps = Vec::new();
        for step in steps {
            let effects = match step.state {
                StepState::Succeeded => &step.success_effects,
                StepState::Failed(_) => &step.failure_effects,
                _ => continue,
            };
            if effects.is_empty() {
                continue;
            }
            let amounts: Result<Vec<_>, _> =
                effects.iter().map(Mutation::reclaimed_bytes).collect();
            let amounts = amounts.map_err(|_| ApprovalError::InvalidInventory)?;
            let reclaimed_bytes = crate::domain::accounting::bytes(amounts)
                .map_err(|_| ApprovalError::InvalidInventory)?;
            applied_steps.push(AppliedStep {
                id: step.id.clone(),
                operations: effects.len(),
                reclaimed_bytes,
            });
        }
        let reclaimed_bytes =
            crate::domain::accounting::bytes(applied_steps.iter().map(|step| step.reclaimed_bytes))
                .map_err(|_| ApprovalError::InvalidInventory)?;
        let report = EffectReport {
            reclaimed_bytes,
            applied_steps,
            succeeded: steps
                .iter()
                .filter(|step| step.state == StepState::Succeeded)
                .count(),
            failed: steps
                .iter()
                .filter(|step| matches!(step.state, StepState::Failed(_)))
                .count(),
            skipped: steps
                .iter()
                .filter(|step| {
                    matches!(
                        step.state,
                        StepState::Skipped(_) | StepState::PolicySkipped(_)
                    )
                })
                .count(),
        };
        apply_effect(&completed, world);
        world.effect_revision = next_revision;
        self.review.plan = completed;
        Ok(report)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedStep {
    pub id: String,
    pub operations: usize,
    pub reclaimed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectReport {
    pub applied_steps: Vec<AppliedStep>,
    pub reclaimed_bytes: u64,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalError {
    WrongPhrase,
    StaleTarget,
    AlreadyConsumed,
    RevisionExhausted,
    InvalidInventory,
}

impl std::fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongPhrase => "confirmation does not match the reviewed target",
            Self::StaleTarget => "target changed · review a new plan",
            Self::AlreadyConsumed => "plan already ran",
            Self::RevisionExhausted => "simulation revision exhausted",
            Self::InvalidInventory => "plan inventory is invalid",
        })
    }
}
impl std::error::Error for ApprovalError {}

#[derive(PartialEq, Eq)]
struct TargetSnapshot {
    host: (
        String,
        crate::domain::host::HostKind,
        crate::domain::host::Environment,
    ),
    cwd: String,
    revision: u64,
    data: TargetData,
}

#[derive(PartialEq, Eq)]
enum TargetData {
    Docker(Option<crate::domain::docker::DockerState>),
    Disk(Option<crate::domain::disk::DiskState>),
    Git(Option<crate::domain::git::GitRepo>),
    Upgrade(
        Option<crate::domain::debian::DebianState>,
        Option<crate::domain::mise::MiseState>,
    ),
    None,
}

impl TargetSnapshot {
    fn capture(plan: &Plan, world: &World) -> Self {
        let data = match plan.effect() {
            Some(PlanEffect::DockerCleanup | PlanEffect::RestartContainer(_)) => {
                TargetData::Docker(world.docker.clone())
            }
            Some(PlanEffect::DiskReclaim) => TargetData::Disk(world.disk.clone()),
            Some(PlanEffect::GitSync) => TargetData::Git(world.git.clone()),
            Some(PlanEffect::DebianUpgraded) => {
                TargetData::Upgrade(world.debian.clone(), world.mise.clone())
            }
            None => TargetData::None,
        };
        Self {
            host: (world.host.name.clone(), world.host.kind, world.host.env),
            cwd: world.cwd.clone(),
            revision: world.effect_revision,
            data,
        }
    }
}

pub fn plan_for(w: &World, action_id: &str) -> Option<ReviewedPlan> {
    let plan = match action_id {
        "docker.cleanup" => docker_cleanup(w),
        "disk.reclaim" => disk_reclaim(w),
        "debian.upgrade" => debian_upgrade(w),
        "git.sync" => git_sync(w),
        id if id.starts_with("docker.restart:") => restart_service(w, &id[15..]),
        _ => None,
    }?;
    Some(ReviewedPlan::new(plan, w))
}

pub fn plan_for_disk_selection(w: &World, selected_paths: &[String]) -> Option<ReviewedPlan> {
    let disk = w.disk.as_ref()?;
    disk.validate(&w.cwd).ok()?;
    if selected_paths.is_empty() {
        return None;
    }

    let mut selected = std::collections::BTreeSet::<&str>::new();
    for path in selected_paths {
        if !selected.insert(path.as_str()) {
            return None;
        }
    }
    let selected_count = disk
        .candidates
        .iter()
        .filter(|candidate| selected.contains(candidate.path.as_str()))
        .count();
    if selected_count != selected.len()
        || disk.candidates.iter().any(|candidate| {
            selected.contains(candidate.path.as_str())
                && candidate.freshness == Freshness::ActiveToday
        })
    {
        return None;
    }

    let plan = disk_reclaim_plan(w, disk, Some(&selected))?;
    Some(ReviewedPlan::new(plan, w))
}

fn apply_effect(plan: &Plan, world: &mut World) {
    for step in plan.steps() {
        let effects = match step.state {
            StepState::Succeeded => &step.success_effects,
            StepState::Failed(_) => &step.failure_effects,
            _ => continue,
        };
        for effect in effects {
            match effect {
                Mutation::RemoveContainers(targets) => {
                    if let Some(docker) = &mut world.docker {
                        docker.containers.retain(|c| !targets.contains(c));
                    }
                }
                Mutation::RemoveImages(targets) => {
                    if let Some(docker) = &mut world.docker {
                        docker.image_inventory.retain(|r| !targets.contains(r));
                    }
                }
                Mutation::RemoveVolumes(targets) => {
                    if let Some(docker) = &mut world.docker {
                        docker.volume_inventory.retain(|r| !targets.contains(r));
                    }
                }
                Mutation::RemoveNetworks(targets) => {
                    if let Some(docker) = &mut world.docker {
                        docker.network_inventory.retain(|r| !targets.contains(r));
                    }
                }
                Mutation::PruneCache(bytes) => {
                    if let Some(docker) = &mut world.docker {
                        docker.cache_bytes = docker.cache_bytes.saturating_sub(*bytes);
                    }
                }
                Mutation::RemoveDisk(target) => {
                    if let Some(disk) = &mut world.disk {
                        disk.candidates.retain(|candidate| candidate != target);
                        disk.used_bytes = disk.used_bytes.saturating_sub(target.size_bytes);
                    }
                }
                Mutation::CheckoutGit {
                    root,
                    branch,
                    upstream,
                } => {
                    if let Some(child) = world
                        .git
                        .as_mut()
                        .and_then(|git| git.children.iter_mut().find(|c| c.root == *root))
                    {
                        child.branch = Some(branch.clone());
                        child.upstream = Some(upstream.clone());
                        child.detached_sha = None;
                    }
                }
                Mutation::FastForwardGit { root, .. } => {
                    if let Some(child) = world
                        .git
                        .as_mut()
                        .and_then(|git| git.children.iter_mut().find(|c| c.root == *root))
                    {
                        child.behind = 0;
                    }
                }
                Mutation::RemoveOrphanPackages(targets) => {
                    if let Some(debian) = &mut world.debian {
                        debian
                            .orphaned_packages
                            .retain(|package| !targets.contains(package));
                    }
                }
                Mutation::UpgradeDebian { held, .. } => {
                    if let Some(debian) = &mut world.debian {
                        debian.pending = *held;
                        debian.security = 0;
                    }
                }
                Mutation::UpgradeTool { name, after, .. } => {
                    if let Some(tool) = world
                        .mise
                        .as_mut()
                        .and_then(|mise| mise.tools.iter_mut().find(|t| t.name == *name))
                    {
                        tool.version.clone_from(after);
                        tool.state = ToolState::Active;
                    }
                }
                Mutation::RestartContainer(target) => {
                    if let Some(container) = world
                        .docker
                        .as_mut()
                        .and_then(|docker| docker.containers.iter_mut().find(|c| *c == target))
                    {
                        container.state = ContainerState::Running;
                        container.health = Some(Health::Healthy);
                    }
                }
            }
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "The complete declarative Docker DAG remains contiguous for dependency and output review"
)]
fn docker_cleanup(w: &World) -> Option<Plan> {
    let d = w.docker.as_ref()?;
    d.validate().ok()?;
    let host = w.host.name.clone();
    let exited: Vec<&str> = d
        .containers
        .iter()
        .filter(|c| c.state == ContainerState::Exited)
        .map(|c| c.name.as_str())
        .collect();
    let mut containers = PlanStep::new(
        "containers",
        "Remove stopped containers",
        "docker container prune -f",
        "containers",
        &[0],
    );

    if exited.contains(&"payments-old") {
        let removed = d
            .containers
            .iter()
            .filter(|c| {
                c.state == ContainerState::Exited && matches!(c.name.as_str(), "web" | "cron")
            })
            .cloned()
            .collect();
        containers = containers.fails_after(
            "payments-old: bind mount still registered · removal refused",
            vec![Mutation::RemoveContainers(removed)],
            "Error: container payments-old: bind mount still registered",
        );
    } else {
        containers = containers.succeeds_with(vec![Mutation::RemoveContainers(
            d.containers
                .iter()
                .filter(|c| c.state == ContainerState::Exited)
                .cloned()
                .collect(),
        )]);
    }
    let steps = vec![
        PlanStep::new(
            "inspect",
            "Inspect Docker usage",
            "docker system df",
            "inspect",
            &[],
        )
        .required()
        .lines(&[
            "TYPE                TOTAL   SIZE",
            &format!(
                "Images              {:<7} {}",
                d.images(),
                human_bytes(d.image_bytes().ok()?)
            ),
            &format!(
                "Containers          {:<7} {}",
                d.containers.len(),
                human_bytes(d.container_bytes().ok()?)
            ),
            &format!(
                "Local Volumes       {:<7} {}",
                d.volumes(),
                human_bytes(d.volume_bytes().ok()?)
            ),
            &format!(
                "Build Cache                 {}",
                human_bytes(d.build_cache_bytes())
            ),
        ]),
        containers,
        PlanStep::new(
            "images",
            "Remove unused images",
            "docker image prune -a -f",
            "images",
            &[1],
        )
        .succeeds_with(vec![Mutation::RemoveImages(
            d.image_inventory
                .iter()
                .filter(|r| r.unused)
                .cloned()
                .collect(),
        )]),
        PlanStep::new(
            "volumes",
            "Remove unused volumes",
            "docker volume prune -f",
            "volumes",
            &[1],
        )
        .succeeds_with(vec![Mutation::RemoveVolumes(
            d.volume_inventory
                .iter()
                .filter(|r| r.unused)
                .cloned()
                .collect(),
        )]),
        PlanStep::new(
            "cache",
            "Prune builder cache",
            "docker builder prune -a -f",
            "builder",
            &[0],
        )
        .succeeds_with(vec![Mutation::PruneCache(d.build_cache_bytes())]),
        PlanStep::new(
            "networks",
            "Remove unused networks",
            "docker network prune -f",
            "networks",
            &[0],
        )
        .succeeds_with(vec![Mutation::RemoveNetworks(
            d.network_inventory
                .iter()
                .filter(|r| r.unused)
                .cloned()
                .collect(),
        )]),
        PlanStep::new(
            "verify",
            "Verify reclaimed space",
            "docker system df",
            "verify",
            &[0],
        )
        .required()
        .lines(&[
            "docker system df",
            "reclaimable recounted · next scan is honest",
        ]),
    ];
    Plan::try_new(PlanSpec {
        action_id: "docker.cleanup".into(),
        title: "Clean up Docker data".into(),
        phrase: format!("REMOVE ALL DOCKER DATA ON {host}"),
        will_change: format!(
            "frees up to {} across containers, images, volumes and builder cache",
            human_bytes(d.reclaimable_bytes().ok()?)
        ),
        host,
        steps,
        effect: Some(PlanEffect::DockerCleanup),
    })
    .ok()
}

fn disk_reclaim(w: &World) -> Option<Plan> {
    let disk = w.disk.as_ref()?;
    disk.validate(&w.cwd).ok()?;
    disk_reclaim_plan(w, disk, None)
}

fn disk_reclaim_plan(
    w: &World,
    disk: &DiskState,
    selected_paths: Option<&std::collections::BTreeSet<&str>>,
) -> Option<Plan> {
    let candidates: Vec<&Candidate> = disk
        .candidates
        .iter()
        .filter(|candidate| {
            selected_paths.is_none_or(|selected| selected.contains(candidate.path.as_str()))
        })
        .collect();
    let selected_subset = selected_paths.is_some();
    let host = w.host.name.clone();
    let mut steps = vec![
        PlanStep::new(
            "preflight",
            "Check filesystem usage",
            "df -h /",
            "inspect",
            &[],
        )
        .required()
        .lines(&[&format!(
            "/dev/sda1  {}  {}  {}% /",
            human_bytes(disk.total_bytes),
            human_bytes(disk.used_bytes),
            disk.used_percent()
        )]),
    ];
    for c in &candidates {
        let title = format!("Remove {}", basename(&c.path));
        let step = PlanStep::new(
            &format!("rm:{}", c.path),
            &title,
            &format!("rm -rf {}", c.path),
            &c.freshness.label(),
            &[0],
        )
        .succeeds_with(vec![Mutation::RemoveDisk((*c).clone())]);
        let step = match c.freshness {
            Freshness::ActiveToday => step.policy_skipped("active today · never removed"),
            _ => step,
        };
        steps.push(step);
    }
    steps.push(
        PlanStep::new("verify", "Verify free space", "df -h /", "verify", &[0])
            .required()
            .lines(&["df -h /", "free space recounted · next scan is honest"]),
    );

    let reclaim = crate::domain::accounting::bytes(
        candidates
            .iter()
            .filter(|candidate| !matches!(candidate.freshness, Freshness::ActiveToday))
            .map(|candidate| candidate.size_bytes),
    )
    .ok()?;
    let (title, will_change) = if selected_subset {
        (
            "Reclaim selected disk space",
            format!(
                "deletes {} across {} selected candidates · unselected candidates stay",
                human_bytes(reclaim),
                candidates.len()
            ),
        )
    } else {
        (
            "Reclaim disk space",
            format!(
                "deletes {} of generated artifacts · active-today artifacts stay",
                human_bytes(reclaim)
            ),
        )
    };
    Plan::try_new(PlanSpec {
        action_id: "disk.reclaim".into(),
        title: title.into(),
        phrase: format!("DELETE GENERATED ARTIFACTS ON {host}"),
        will_change,
        host,
        steps,
        effect: Some(PlanEffect::DiskReclaim),
    })
    .ok()
}

#[expect(
    clippy::too_many_lines,
    reason = "The complete declarative upgrade DAG remains contiguous for dependency and output review"
)]
fn debian_upgrade(w: &World) -> Option<Plan> {
    let deb = w.debian.as_ref()?;
    deb.validate().ok()?;
    let host = w.host.name.clone();
    let outdated: Vec<String> = w
        .mise
        .as_ref()
        .map(|m| {
            m.tools
                .iter()
                .filter_map(|t| match &t.state {
                    ToolState::Outdated { latest } => {
                        Some(format!("{} {} → {}", t.name, t.version, latest))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let upgraded = deb.pending.checked_sub(deb.held)?;
    if deb.security > upgraded {
        return None;
    }
    let tools = w
        .mise
        .as_ref()
        .map_or_else(|| [].as_slice(), |mise| mise.tools.as_slice());
    let mut tool_ids = std::collections::BTreeSet::new();
    if tools
        .iter()
        .any(|tool| tool.name.is_empty() || !tool_ids.insert(tool.name.as_str()))
    {
        return None;
    }
    let tool_effects: Vec<Mutation> = tools
        .iter()
        .filter_map(|tool| match &tool.state {
            ToolState::Outdated { latest } => Some(Mutation::UpgradeTool {
                name: tool.name.clone(),
                before: tool.version.clone(),
                after: latest.clone(),
            }),
            _ => None,
        })
        .collect();
    let tool_lines: Vec<String> = tools
        .iter()
        .map(|tool| match &tool.state {
            ToolState::Outdated { latest } => {
                format!("{:<8}{}  latest {latest}", tool.name, tool.version)
            }
            ToolState::Missing => {
                format!("{:<8}{}  missing (not required)", tool.name, tool.version)
            }
            ToolState::Active => format!("{:<8}{}  current", tool.name, tool.version),
        })
        .collect();
    let command = if tool_effects.is_empty() {
        "mise up".into()
    } else {
        format!(
            "mise up {}",
            tools
                .iter()
                .filter_map(|tool| match &tool.state {
                    ToolState::Outdated { latest } => Some(format!("{}@{latest}", tool.name)),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ")
        )
    };
    let mut inspect_tools = PlanStep::new(
        "mise-inspect",
        "Inspect tool versions",
        "mise ls",
        "mise",
        &[0],
    )
    .required()
    .lines(&tool_lines.iter().map(String::as_str).collect::<Vec<_>>());
    let mut upgrade_tools = PlanStep::new(
        "mise-upgrade",
        "Upgrade outdated tools",
        &command,
        "mise",
        &[4],
    )
    .succeeds_with(tool_effects);
    if w.mise.is_none() {
        inspect_tools = inspect_tools.policy_skipped("mise is not configured on this host");
        upgrade_tools = upgrade_tools.policy_skipped("mise is not configured on this host");
    }
    let mut verify = vec![format!("0 upgrades pending · {} held back", deb.held)];
    if w.mise.is_some() {
        let missing = tools
            .iter()
            .filter(|tool| tool.state == ToolState::Missing)
            .count();
        verify.push(if missing == 0 {
            "all tools current".into()
        } else {
            format!("installed tools current · {missing} optional tools missing")
        });
    }
    verify.push(if deb.reboot_required {
        "reboot still required · schedule separately".into()
    } else {
        "no reboot required".into()
    });
    let verify_deps = if w.mise.is_some() {
        vec![2, 5]
    } else {
        vec![2]
    };
    let steps = vec![
        PlanStep::new(
            "preflight",
            "Preflight checks",
            "uname -a && df -h /",
            "preflight",
            &[],
        )
        .required()
        .lines(&[
            &format!("Linux {host} 6.1.0-31-amd64 x86_64"),
            "disk headroom sufficient · load average normal",
        ]),
        PlanStep::new(
            "apt-update",
            "Refresh package metadata",
            "apt update",
            "apt",
            &[0],
        )
        .required()
        .lines(&[
            "Hit:12 bookworm-security InRelease",
            &format!("{} packages can be upgraded", deb.pending),
        ]),
        PlanStep::new(
            "apt-upgrade",
            "Apply upgrades",
            "apt full-upgrade -y",
            "apt",
            &[1],
        )
        .succeeds_with(vec![Mutation::UpgradeDebian {
            upgraded,
            security: deb.security,
            held: deb.held,
        }]),
        PlanStep::new(
            "autoremove",
            "Remove orphaned packages",
            "apt autoremove -y",
            "apt",
            &[2],
        )
        .succeeds_with(vec![Mutation::RemoveOrphanPackages(
            deb.orphaned_packages.clone(),
        )]),
        inspect_tools,
        upgrade_tools,
        PlanStep::new(
            "verify",
            "Verify the host",
            "apt list --upgradable && mise ls",
            "verify",
            &verify_deps,
        )
        .required()
        .lines(&verify.iter().map(String::as_str).collect::<Vec<_>>()),
    ];
    Plan::try_new(PlanSpec {
        action_id: "debian.upgrade".into(),
        title: "Upgrade everything on this host".into(),
        phrase: format!("I UNDERSTAND: UPGRADE EVERYTHING ON {host}"),
        will_change: format!(
            "{} upgrades ({} security){}{}",
            upgraded,
            deb.security,
            if outdated.is_empty() {
                String::new()
            } else {
                format!(" + {}", outdated.join(", "))
            },
            if deb.reboot_required {
                " · reboot required after"
            } else {
                ""
            }
        ),
        host,
        steps,
        effect: Some(PlanEffect::DebianUpgraded),
    })
    .ok()
}

#[expect(
    clippy::too_many_lines,
    reason = "The per-child declarative Git DAG remains contiguous for dependency and policy review"
)]
fn git_sync(w: &World) -> Option<Plan> {
    let git = w.git.as_ref()?;
    if git.children.is_empty() {
        return None;
    }
    let host = w.host.name.clone();
    let resolved: Vec<String> = git
        .children
        .iter()
        .map(|c| format!("{} · primary {}", c.root, c.primary_branch))
        .collect();
    let mut steps = vec![
        PlanStep::new(
            "preflight",
            "Resolve child repositories",
            "git -C <child> rev-parse --show-toplevel",
            "inspect",
            &[],
        )
        .required()
        .lines(&resolved.iter().map(String::as_str).collect::<Vec<_>>()),
    ];
    for c in &git.children {
        let name = basename(&c.root);
        let fetch_i = steps.len();
        if c.detached() {
            steps.push(
                PlanStep::new(
                    &format!("pull:{}", c.root),
                    &format!("Update {name}"),
                    &format!("git -C {} pull --ff-only", c.root),
                    name,
                    &[0],
                )
                .policy_skipped(&format!(
                    "detached at {} · skipped",
                    c.detached_sha.as_deref().unwrap_or("?")
                )),
            );
            continue;
        }
        steps.push(
            PlanStep::new(
                &format!("fetch:{}", c.root),
                &format!("Fetch {name}"),
                &format!("git -C {} fetch --prune", c.root),
                name,
                &[0],
            )
            .lines(&[&format!("fetch origin · {name} up to date refs")]),
        );
        let checkout_i = steps.len();
        steps.push(
            PlanStep::new(
                &format!("checkout:{}", c.root),
                &format!("Check out {} ({name})", c.primary_branch),
                &format!("git -C {} checkout {}", c.root, c.primary_branch),
                name,
                &[fetch_i],
            )
            .succeeds_with(vec![Mutation::CheckoutGit {
                root: c.root.clone(),
                branch: c.primary_branch.clone(),
                upstream: format!("origin/{}", c.primary_branch),
            }]),
        );
        let pull = PlanStep::new(
            &format!("pull:{}", c.root),
            &format!("Pull {name} — fast-forward only"),
            &format!("git -C {} pull --ff-only", c.root),
            name,
            &[checkout_i],
        );

        let pull = if c.diverged() {
            pull.fails_with(
                &format!(
                    "diverged · {} ahead, {} behind · needs manual merge",
                    c.ahead, c.behind
                ),
                &[
                    "hint: You have divergent branches",
                    "fatal: Not possible to fast-forward, aborting",
                ],
            )
        } else {
            pull.succeeds_with(vec![Mutation::FastForwardGit {
                root: c.root.clone(),
                branch: c.primary_branch.clone(),
                commits: c.behind,
            }])
        };
        steps.push(pull);
        if c.diverged() {
            for step in steps.iter_mut().skip(fetch_i) {
                step.state = StepState::PolicySkipped(format!(
                    "diverged · {} ahead, {} behind · untouched",
                    c.ahead, c.behind
                ));
                step.optional = false;
            }
        }
    }
    let behind_total: u32 = git
        .children
        .iter()
        .filter(|c| !c.diverged() && !c.detached())
        .map(|c| c.behind)
        .sum();
    Plan::try_new(PlanSpec {
        action_id: "git.sync".into(),
        title: "Update all child projects".into(),
        phrase: format!("UPDATE ALL CHILD PROJECTS IN {}", git.root),
        will_change: format!(
            "{} child repositories · {} commits to fast-forward · diverged and detached children stay untouched",
            git.children.len(),
            behind_total
        ),
        host,
        steps,
        effect: Some(PlanEffect::GitSync),
    }).ok()
}

fn restart_service(w: &World, name: &str) -> Option<Plan> {
    let d = w.docker.as_ref()?;
    d.validate().ok()?;
    let c = d.containers.iter().find(|c| c.name == name)?;
    let host = w.host.name.clone();
    let steps = vec![
        PlanStep::new(
            "inspect",
            &format!("Inspect {name}"),
            &format!("docker inspect {name}"),
            "inspect",
            &[],
        )
        .required()
        .lines(&[
            &format!("image {}", c.image),
            "status restarting · health unhealthy",
            "restart count 14 · OOM kills 0",
        ]),
        PlanStep::new(
            "restart",
            &format!("Restart {name}"),
            &format!("docker restart {name}"),
            "restart",
            &[0],
        )
        .required()
        .succeeds_with(vec![Mutation::RestartContainer(c.clone())]),
        PlanStep::new(
            "verify",
            "Verify health",
            &format!("docker ps --filter name={name}"),
            "verify",
            &[1],
        )
        .required()
        .lines(&[&format!("{name}  up 4 seconds (healthy)")]),
    ];
    Plan::try_new(PlanSpec {
        action_id: format!("docker.restart:{name}"),
        title: format!("Restart {name}"),
        phrase: format!("RESTART {} ON {}", name.to_uppercase(), host),
        will_change: format!(
            "restarts {name} ({}) on {host} · brief downtime for the service",
            c.image
        ),
        host,
        steps,
        effect: Some(PlanEffect::RestartContainer(name.into())),
    })
    .ok()
}

fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}
