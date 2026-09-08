//! Plan builders: turn a Broad catalogue action into a reviewable DAG of
//! fixture-deterministic steps, and apply the honest world mutation after a
//! run. Phrases are bound to the target host (`REMOVE ALL DOCKER DATA ON
//! devbox`); the broadest plans demand the `I UNDERSTAND:` prefix.

use crate::domain::disk::Freshness;
use crate::domain::docker::{ContainerState, Health};
use crate::domain::effect::Mutation;
use crate::domain::human_bytes;
use crate::domain::mise::ToolState;
use crate::domain::plan::{Plan, PlanEffect, PlanSpec, PlanStep, StepState};
use crate::sim::world::World;

/// A review bound to the exact simulated host and effect target facts.
/// It cannot be cloned, reconstructed from UI fields, or executed twice.
pub(crate) struct ReviewedPlan {
    plan: Plan,
    target: TargetSnapshot,
}

impl ReviewedPlan {
    fn new(plan: Plan, world: &World) -> Self {
        let target = TargetSnapshot::capture(&plan, world);
        Self { plan, target }
    }

    pub(crate) fn plan(&self) -> &Plan {
        &self.plan
    }

    pub(crate) fn toggle(
        &mut self,
        index: usize,
    ) -> Result<String, crate::domain::plan::PlanError> {
        self.plan.toggle(index)
    }

    /// Bind an exact typed confirmation to this review and current target.
    ///
    /// # Errors
    /// Refuses a wrong phrase, an already-consumed review or changed target.
    /// The phrase is never retained or included in the error.
    pub(crate) fn approve<'a>(
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
        if self.target != TargetSnapshot::capture(&self.plan, world) {
            return Err(ApprovalError::StaleTarget);
        }
        Ok(())
    }
}

/// Exclusive, one-use permission to apply this in-memory simulation.
/// Dropping approval cancels it without running or consuming the review.
pub(crate) struct Approval<'a> {
    review: &'a mut ReviewedPlan,
}

impl Approval<'_> {
    /// Revalidate immediately before running and applying deterministic effects.
    ///
    /// # Errors
    /// A changed target refuses the whole run with no model mutation. Successful
    /// execution consumes the review even when fixture steps fail partway.
    pub(crate) fn execute(self, world: &mut World) -> Result<EffectReport, ApprovalError> {
        self.review.validate(world)?;
        let next_revision = world
            .effect_revision
            .checked_add(1)
            .ok_or(ApprovalError::RevisionExhausted)?;
        // Simulate completion on a private candidate first. Invalid accounting
        // never consumes the review or mutates the world.
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

/// Executed mutation groups, including truthful partial failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppliedStep {
    pub(crate) id: String,
    pub(crate) operations: usize,
    pub(crate) reclaimed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EffectReport {
    pub(crate) applied_steps: Vec<AppliedStep>,
    pub(crate) reclaimed_bytes: u64,
    pub(crate) succeeded: usize,
    pub(crate) failed: usize,
    pub(crate) skipped: usize,
}

/// Failure codes never contain typed confirmation text or target payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ApprovalError {
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

// Only effect-relevant facts participate; discovery time and unrelated UI
// activity cannot invalidate a review. Exact value comparison detects direct
// fixture edits, while the revision detects intervening simulated commits.
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

/// The plan behind a Broad action, if this world has one.
pub(crate) fn plan_for(w: &World, action_id: &str) -> Option<ReviewedPlan> {
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

/// Apply only the mutations attached to the actual terminal step outcome.
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

/// Complete Docker cleanup: inspection, the containers → images/volumes
/// chain, builder cache and networks as parallel branches, verification.
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
    // fixture truth: payments-old still has a bind mount registered, so the
    // removal fails partway — dependents (images, volumes) must propagate
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

/// Disk reclaim: one removal step per inactive/unknown candidate; artifacts
/// in use today are policy-skipped and shown, never removed.
fn disk_reclaim(w: &World) -> Option<Plan> {
    let disk = w.disk.as_ref()?;
    disk.validate(&w.cwd).ok()?;
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
    for c in &disk.candidates {
        let title = format!("Remove {}", basename(&c.path));
        let step = PlanStep::new(
            &format!("rm:{}", c.path),
            &title,
            &format!("rm -rf {}", c.path),
            &c.freshness.label(),
            &[0],
        )
        .succeeds_with(vec![Mutation::RemoveDisk(c.clone())]);
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
    // reclaim excludes what policy keeps: active-today artifacts stay
    let reclaim: u64 = disk
        .candidates
        .iter()
        .filter(|c| !matches!(c.freshness, Freshness::ActiveToday))
        .map(|c| c.size_bytes)
        .sum();
    Plan::try_new(PlanSpec {
        action_id: "disk.reclaim".into(),
        title: "Reclaim disk space".into(),
        phrase: format!("DELETE GENERATED ARTIFACTS ON {host}"),
        will_change: format!(
            "deletes {} of generated artifacts · active-today artifacts stay",
            human_bytes(reclaim)
        ),
        host,
        steps,
        effect: Some(PlanEffect::DiskReclaim),
    })
    .ok()
}

/// Upgrade everything on a Debian host: apt branch and mise branch in
/// parallel, optional cleanup, final verification (CONCEPT §8.16).
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
    if deb.held > deb.pending || deb.security > deb.pending - deb.held {
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
            upgraded: deb.pending - deb.held,
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
            deb.pending - deb.held,
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

/// Update every nested child repo: one fetch→checkout-primary→pull chain
/// per child, all parallel off a shared preflight. The primary branch is
/// resolved per child, never assumed; diverged children fail their pull and
/// detached children are policy-skipped, shown, never touched.
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
            &[fetch_i + 1],
        );
        // fixture truth: a diverged child cannot fast-forward; the failure
        // is per-child and never blocks sibling branches
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
            for step in &mut steps[fetch_i..] {
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

/// Restart one unhealthy service: inspect → restart → verify. On a
/// production host this is Broad: two gates, typed phrase.
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

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "Tests assert fixed fixture structure and bounded values; violations must fail the test"
)]
mod tests {
    use super::*;
    use crate::domain::fixtures;
    use crate::scenario::Scenario;

    fn settled(s: Scenario) -> World {
        let mut w = fixtures::world_for(s);
        w.clock.running = true;
        w.seek(4_000);
        w
    }

    #[test]
    fn docker_plan_binds_phrase_and_propagates_failure() {
        let mut w = settled(Scenario::DockerCleanup);
        let mut p = plan_for(&w, "docker.cleanup").unwrap();
        assert_eq!(p.plan().phrase(), "REMOVE ALL DOCKER DATA ON devbox");
        let phrase = p.plan().phrase().to_owned();
        p.approve(&phrase, &w).unwrap().execute(&mut w).unwrap();
        assert!(matches!(p.plan().steps()[1].state, StepState::Failed(_)));
        assert!(matches!(p.plan().steps()[2].state, StepState::Skipped(_)));
        assert!(matches!(p.plan().steps()[4].state, StepState::Succeeded));
        let d = w.docker.unwrap();
        assert_eq!(d.build_cache_bytes(), 0);
        assert!(d.containers.iter().any(|c| c.name == "payments-old"));
        assert!(!d.containers.iter().any(|c| c.name == "web"));
    }

    #[test]
    fn debian_plan_has_parallel_branches_and_policy_phrase() {
        let mut w = settled(Scenario::UpgradePlan);
        let mut p = plan_for(&w, "debian.upgrade").unwrap();
        assert_eq!(
            p.plan().phrase(),
            "I UNDERSTAND: UPGRADE EVERYTHING ON devbox-deb"
        );
        p.toggle(2).unwrap(); // exclude Apply upgrades
        assert!(
            p.plan().blocked_by_exclusion(3).is_some(),
            "autoremove blocked"
        );
        assert!(p.plan().blocked_by_exclusion(6).is_some(), "verify blocked");
        assert_eq!(
            p.plan().blocked_by_exclusion(5),
            None,
            "mise branch parallel"
        );
        let phrase = p.plan().phrase().to_owned();
        p.approve(&phrase, &w).unwrap().execute(&mut w).unwrap();
        assert!(matches!(p.plan().steps()[5].state, StepState::Succeeded));
        assert_eq!(w.debian.as_ref().unwrap().pending, 47, "apt not applied");
        let mise = w.mise.as_ref().unwrap();
        assert!(
            mise.tools
                .iter()
                .all(|t| !matches!(t.state, ToolState::Outdated { .. }))
        );
    }

    #[test]
    fn restart_phrase_uses_uppercase_service_and_host() {
        let mut w = settled(Scenario::RemoteHost);
        let mut p = plan_for(&w, "docker.restart:payments").unwrap();
        assert_eq!(p.plan().phrase(), "RESTART PAYMENTS ON prod-eu-1");
        let phrase = p.plan().phrase().to_owned();
        p.approve(&phrase, &w).unwrap().execute(&mut w).unwrap();
        let d = w.docker.as_ref().unwrap();
        assert!(d.unhealthy().is_empty());
    }

    #[test]
    fn disk_plan_policy_skips_active_today() {
        let w = settled(Scenario::DiskCleanup);
        let p = plan_for(&w, "disk.reclaim").unwrap();
        assert_eq!(p.plan().phrase(), "DELETE GENERATED ARTIFACTS ON devbox");
        let active = p
            .plan()
            .steps()
            .iter()
            .find(|s| matches!(s.state, StepState::PolicySkipped(_)))
            .unwrap();
        assert!(active.title.contains("node_modules"));
    }
    #[test]
    fn approval_revalidates_target_after_phrase_before_any_effect() {
        let mut world = settled(Scenario::DiskCleanup);
        let mut review = plan_for(&world, "disk.reclaim").unwrap();
        let phrase = review.plan().phrase().to_owned();
        let approval = review.approve(&phrase, &world).unwrap();
        world.disk.as_mut().unwrap().candidates[0].freshness = Freshness::ActiveToday;
        let before = world.disk.clone();
        assert_eq!(
            approval.execute(&mut world),
            Err(ApprovalError::StaleTarget)
        );
        assert_eq!(world.disk, before);
        assert!(!review.plan().ran());
        assert_eq!(world.effect_revision, 0);
    }

    #[test]
    fn approval_checks_phrase_host_and_revision_without_consuming_review() {
        let mut world = settled(Scenario::RemoteHost);
        let mut review = plan_for(&world, "docker.restart:payments").unwrap();
        assert!(matches!(
            review.approve("wrong", &world),
            Err(ApprovalError::WrongPhrase)
        ));
        let phrase = review.plan().phrase().to_owned();
        let old_host = world.host.clone();
        world.host.env = crate::domain::host::Environment::Local;
        assert!(matches!(
            review.approve(&phrase, &world),
            Err(ApprovalError::StaleTarget)
        ));
        world.host = old_host;
        world.effect_revision += 1;
        assert!(matches!(
            review.approve(&phrase, &world),
            Err(ApprovalError::StaleTarget)
        ));
        assert!(!review.plan().ran());
    }

    #[test]
    fn successful_and_partially_failed_reviews_are_one_shot() {
        for (scenario, action, expected_failures) in [
            (Scenario::RemoteHost, "docker.restart:payments", 0),
            (Scenario::DockerCleanup, "docker.cleanup", 1),
        ] {
            let mut world = settled(scenario);
            let mut review = plan_for(&world, action).unwrap();
            let phrase = review.plan().phrase().to_owned();
            let report = review
                .approve(&phrase, &world)
                .unwrap()
                .execute(&mut world)
                .unwrap();
            assert_eq!(report.failed, expected_failures);
            let after = world.docker.clone();
            assert!(matches!(
                review.approve(&phrase, &world),
                Err(ApprovalError::AlreadyConsumed)
            ));
            assert_eq!(world.docker, after);
            assert_eq!(world.effect_revision, 1);
            assert!(review.plan().ran());
        }
    }

    #[test]
    fn cancel_approval_keeps_world_and_review_unchanged() {
        let mut world = settled(Scenario::DockerCleanup);
        let mut review = plan_for(&world, "docker.cleanup").unwrap();
        let before = world.docker.clone();
        let phrase = review.plan().phrase().to_owned();
        {
            let _approval = review.approve(&phrase, &world).unwrap();
        }
        assert_eq!(world.docker, before);
        assert!(!review.plan().ran());
        world.tick(1000);
        assert!(
            review.approve(&phrase, &world).is_ok(),
            "elapsed discovery time is not a target change"
        );
    }

    #[test]
    fn approval_refuses_revision_exhaustion_before_running() {
        let mut world = settled(Scenario::DockerCleanup);
        world.effect_revision = u64::MAX;
        let mut review = plan_for(&world, "docker.cleanup").unwrap();
        let before = world.docker.clone();
        let phrase = review.plan().phrase().to_owned();
        assert_eq!(
            review.approve(&phrase, &world).unwrap().execute(&mut world),
            Err(ApprovalError::RevisionExhausted)
        );
        assert_eq!(world.docker, before);
        assert!(!review.plan().ran());
    }
    const MIB: u64 = 1_024 * 1_024;

    fn execute_review(review: &mut ReviewedPlan, world: &mut World) -> EffectReport {
        let phrase = review.plan().phrase().to_owned();
        review
            .approve(&phrase, world)
            .unwrap()
            .execute(world)
            .unwrap()
    }

    #[test]
    fn docker_partial_failure_accounts_exact_removed_targets() {
        let mut world = settled(Scenario::DockerCleanup);
        let mut review = plan_for(&world, "docker.cleanup").unwrap();
        let report = execute_review(&mut review, &mut world);
        let docker = world.docker.as_ref().unwrap();
        assert_eq!(docker.container_bytes().unwrap(), 340 * MIB);
        assert_eq!(
            (docker.images(), docker.image_bytes().unwrap()),
            (9, 6_200 * MIB)
        );
        assert_eq!(
            (docker.volumes(), docker.volume_bytes().unwrap()),
            (5, 3_400 * MIB)
        );
        assert_eq!(docker.networks(), 4);
        assert_eq!(report.reclaimed_bytes, 12_780 * MIB);
        assert_eq!(
            report
                .applied_steps
                .iter()
                .map(|step| step.id.as_str())
                .collect::<Vec<_>>(),
            ["containers", "cache", "networks"]
        );
        assert_eq!(
            review.plan().steps()[1].lines,
            [
                "Removed web",
                "Removed cron",
                "Error: container payments-old: bind mount still registered"
            ]
        );
    }

    #[test]
    fn docker_success_recounts_resources_and_output_from_same_inventory() {
        let mut world = settled(Scenario::DockerCleanup);
        world
            .docker
            .as_mut()
            .unwrap()
            .containers
            .retain(|c| c.name != "payments-old");
        let before = world.docker.as_ref().unwrap().total_bytes().unwrap();
        let mut review = plan_for(&world, "docker.cleanup").unwrap();
        let report = execute_review(&mut review, &mut world);
        let docker = world.docker.as_ref().unwrap();
        assert_eq!(
            (docker.images(), docker.image_bytes().unwrap()),
            (5, 3_100 * MIB)
        );
        assert_eq!((docker.volumes(), docker.volume_bytes().unwrap()), (0, 0));
        assert_eq!(docker.container_bytes().unwrap(), 300 * MIB);
        assert_eq!(docker.networks(), 4);
        assert_eq!(
            before - docker.total_bytes().unwrap(),
            report.reclaimed_bytes
        );
        assert_eq!(
            review.plan().steps()[2].lines,
            ["Deleted 4 images", "Total reclaimed: 3.0 GB"]
        );
    }

    #[test]
    fn excluded_cleanup_steps_have_no_hidden_effects() {
        let mut world = settled(Scenario::DockerCleanup);
        let before = world.docker.as_ref().unwrap().total_bytes().unwrap();
        let mut review = plan_for(&world, "docker.cleanup").unwrap();
        review.toggle(1).unwrap();
        review.toggle(4).unwrap();
        let report = execute_review(&mut review, &mut world);
        assert_eq!(
            world.docker.as_ref().unwrap().total_bytes().unwrap(),
            before
        );
        assert_eq!(report.reclaimed_bytes, 0);
        assert_eq!(world.docker.as_ref().unwrap().containers.len(), 7);
    }

    #[test]
    fn disk_claims_only_policy_removable_bytes_and_accounts_exactly() {
        let mut world = settled(Scenario::DiskCleanup);
        let disk = world.disk.as_ref().unwrap();
        assert_eq!(disk.inspected_bytes().unwrap(), 26_040 * MIB);
        assert_eq!(disk.reclaimable_bytes().unwrap(), 22_940 * MIB);
        let before = disk.used_bytes;
        let mut review = plan_for(&world, "disk.reclaim").unwrap();
        let report = execute_review(&mut review, &mut world);
        let disk = world.disk.as_ref().unwrap();
        assert_eq!(disk.used_bytes, before - 22_940 * MIB);
        assert_eq!(report.reclaimed_bytes, 22_940 * MIB);
        assert_eq!(disk.candidates.len(), 1);
        assert_eq!(disk.candidates[0].freshness, Freshness::ActiveToday);
    }

    #[test]
    fn git_checkout_is_real_in_model_and_diverged_children_stay_untouched() {
        let mut world = settled(Scenario::MonorepoRoot);
        let children = &mut world.git.as_mut().unwrap().children;
        children[0].branch = Some("feature".into());
        children[0].upstream = Some("origin/feature".into());
        let diverged = children[1].clone();
        let detached = children[2].clone();
        let mut review = plan_for(&world, "git.sync").unwrap();
        execute_review(&mut review, &mut world);
        let children = &world.git.as_ref().unwrap().children;
        assert_eq!(children[0].branch.as_deref(), Some("main"));
        assert_eq!(children[0].upstream.as_deref(), Some("origin/main"));
        assert_eq!(children[0].behind, 0);
        assert_eq!(children[1], diverged);
        assert_eq!(children[2], detached);
        assert!(
            review
                .plan()
                .steps()
                .iter()
                .filter(|s| s.id.contains("services/billing"))
                .all(|s| matches!(s.state, StepState::PolicySkipped(_)))
        );
    }

    #[test]
    fn debian_remote_output_does_not_invent_tools_held_packages_or_reboot() {
        let mut world = settled(Scenario::RemoteHost);
        let mut review = plan_for(&world, "debian.upgrade").unwrap();
        execute_review(&mut review, &mut world);
        let lines: Vec<&str> = review
            .plan()
            .steps()
            .iter()
            .flat_map(|s| s.lines.iter().map(String::as_str))
            .collect();
        assert!(lines.contains(&"12 upgraded, 3 security, 0 held back"));
        assert!(lines.contains(&"no reboot required"));
        assert!(
            !lines
                .iter()
                .any(|line| line.contains("rust") || line.contains("kernel"))
        );
        assert!(world.mise.is_none());
        assert_eq!(world.debian.as_ref().unwrap().pending, 0);
    }

    #[test]
    fn upgrade_fixture_retains_orphan_accounting_and_missing_optional_tool_truth() {
        let mut world = settled(Scenario::UpgradePlan);
        let mut review = plan_for(&world, "debian.upgrade").unwrap();
        let report = execute_review(&mut review, &mut world);
        assert_eq!(
            review.plan().steps()[3].lines,
            ["Removing 12 orphaned packages", "Freed 410 MB"]
        );
        assert!(world.debian.as_ref().unwrap().orphaned_packages.is_empty());
        assert_eq!(report.reclaimed_bytes, 410 * MIB);
        assert!(
            review.plan().steps()[6]
                .lines
                .iter()
                .any(|line| line == "installed tools current · 1 optional tools missing")
        );
        assert_eq!(world.debian.as_ref().unwrap().pending, 1);
    }
    #[test]
    fn duplicate_tool_identity_refuses_review_before_any_upgrade() {
        let mut world = settled(Scenario::UpgradePlan);
        let tools = &mut world.mise.as_mut().unwrap().tools;
        let mut duplicate = tools[0].clone();
        duplicate.version = "different-before".into();
        tools.push(duplicate);
        let before = world.mise.clone();
        assert!(plan_for(&world, "debian.upgrade").is_none());
        assert_eq!(world.mise, before);
        assert_eq!(world.effect_revision, 0);
    }

    #[test]
    fn inconsistent_disk_capacity_refuses_reclaim_claims() {
        let mut world = settled(Scenario::DiskCleanup);
        world.disk.as_mut().unwrap().used_bytes = 1;
        assert!(plan_for(&world, "disk.reclaim").is_none());
        world.disk.as_mut().unwrap().used_bytes = u64::MAX;
        assert!(plan_for(&world, "disk.reclaim").is_none());
    }
    #[test]
    fn disk_relative_and_parent_aliases_cannot_double_count_targets() {
        for (first, second) in [(".", "a"), ("/a/x/../b", "/a/b")] {
            let mut world = settled(Scenario::DiskCleanup);
            let disk = world.disk.as_mut().unwrap();
            disk.candidates.truncate(2);
            disk.candidates[0].path = first.into();
            disk.candidates[1].path = second.into();
            assert!(plan_for(&world, "disk.reclaim").is_none());
        }
        let mut world = settled(Scenario::DiskCleanup);
        let disk = world.disk.as_mut().unwrap();
        disk.total_bytes = u64::MAX;
        disk.used_bytes = u64::MAX;
        assert!(plan_for(&world, "disk.reclaim").is_some());
    }
    #[test]
    fn malformed_docker_getters_and_catalogue_report_errors_without_panics() {
        use crate::domain::accounting::InventoryError;
        let mut world = settled(Scenario::DockerCleanup);
        world.docker.as_mut().unwrap().containers[0].size_bytes = u64::MAX;
        let docker = world.docker.as_ref().unwrap();
        assert_eq!(docker.container_bytes(), Err(InventoryError::Overflow));
        assert_eq!(docker.total_bytes(), Err(InventoryError::Overflow));
        assert!(plan_for(&world, "docker.cleanup").is_none());
        let note = crate::sim::catalogue::catalogue(&world)
            .into_iter()
            .find(|row| row.id == "docker.invalid")
            .unwrap();
        assert!(matches!(
            note.availability,
            crate::domain::action::Availability::Blocked(_)
        ));
    }

    #[test]
    fn network_storage_claims_refuse_display_and_review() {
        let mut world = settled(Scenario::DockerCleanup);
        let docker = world.docker.as_mut().unwrap();
        docker.network_inventory[0].size_bytes = 123;
        assert_eq!(
            docker.total_bytes(),
            Err(crate::domain::accounting::InventoryError::NetworkBytes)
        );
        assert_eq!(
            docker.reclaimable_bytes(),
            Err(crate::domain::accounting::InventoryError::NetworkBytes)
        );
        assert!(plan_for(&world, "docker.cleanup").is_none());
    }

    #[test]
    fn orphan_overflow_and_duplicate_resource_ids_refuse_review() {
        let mut world = settled(Scenario::UpgradePlan);
        world.debian.as_mut().unwrap().orphaned_packages[0].size_bytes = u64::MAX;
        assert!(plan_for(&world, "debian.upgrade").is_none());
        let note = crate::sim::catalogue::catalogue(&world)
            .into_iter()
            .find(|row| row.id == "debian.invalid")
            .unwrap();
        assert!(matches!(
            note.availability,
            crate::domain::action::Availability::Blocked(_)
        ));
        let mut world = settled(Scenario::UpgradePlan);
        let packages = &mut world.debian.as_mut().unwrap().orphaned_packages;
        packages[1].id = packages[0].id.clone();
        assert!(plan_for(&world, "debian.upgrade").is_none());
        let mut world = settled(Scenario::DockerCleanup);
        let images = &mut world.docker.as_mut().unwrap().image_inventory;
        images[1].id = images[0].id.clone();
        assert!(plan_for(&world, "docker.cleanup").is_none());
    }

    #[test]
    fn invalid_combined_effect_report_never_consumes_review_or_world() {
        let mut world = settled(Scenario::DockerCleanup);
        let plan = Plan::try_new(PlanSpec {
            action_id: "test".into(),
            title: "test".into(),
            host: world.host.name.clone(),
            phrase: "CONFIRM".into(),
            will_change: "test".into(),
            effect: None,
            steps: vec![
                PlanStep::new("a", "A", "fixture", "a", &[])
                    .succeeds_with(vec![Mutation::PruneCache(u64::MAX)]),
                PlanStep::new("b", "B", "fixture", "b", &[])
                    .succeeds_with(vec![Mutation::PruneCache(u64::MAX)]),
            ],
        })
        .unwrap();
        let mut review = ReviewedPlan::new(plan, &world);
        let before = world.docker.clone();
        assert_eq!(
            review
                .approve("CONFIRM", &world)
                .unwrap()
                .execute(&mut world),
            Err(ApprovalError::InvalidInventory)
        );
        assert_eq!(world.docker, before);
        assert!(!review.plan().ran());
        assert_eq!(world.effect_revision, 0);
    }
}
