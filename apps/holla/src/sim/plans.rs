//! Plan builders: turn a Broad catalogue action into a reviewable DAG of
//! fixture-deterministic steps, and apply the honest world mutation after a
//! run. Phrases are bound to the target host (`REMOVE ALL DOCKER DATA ON
//! devbox`); the broadest plans demand the `I UNDERSTAND:` prefix.

use crate::domain::disk::Freshness;
use crate::domain::docker::{ContainerState, Health};
use crate::domain::human_bytes;
use crate::domain::mise::ToolState;
use crate::domain::plan::{Plan, PlanEffect, PlanSpec, PlanStep, StepState};
use crate::sim::world::World;

/// A review bound to the exact simulated host and effect target facts.
/// It cannot be cloned, reconstructed from UI fields, or executed twice.
pub struct ReviewedPlan {
    plan: Plan,
    target: TargetSnapshot,
}

impl ReviewedPlan {
    fn new(plan: Plan, world: &World) -> Self {
        let target = TargetSnapshot::capture(&plan, world);
        Self { plan, target }
    }

    pub fn plan(&self) -> &Plan {
        &self.plan
    }

    pub fn toggle(&mut self, index: usize) -> Result<String, crate::domain::plan::PlanError> {
        self.plan.toggle(index)
    }

    /// Bind an exact typed confirmation to this review and current target.
    ///
    /// # Errors
    /// Refuses a wrong phrase, an already-consumed review or changed target.
    /// The phrase is never retained or included in the error.
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
        if self.target != TargetSnapshot::capture(&self.plan, world) {
            return Err(ApprovalError::StaleTarget);
        }
        Ok(())
    }
}

/// Exclusive, one-use permission to apply this in-memory simulation.
/// Dropping approval cancels it without running or consuming the review.
pub struct Approval<'a> {
    review: &'a mut ReviewedPlan,
}

impl Approval<'_> {
    /// Revalidate immediately before running and applying deterministic effects.
    ///
    /// # Errors
    /// A changed target refuses the whole run with no model mutation. Successful
    /// execution consumes the review even when fixture steps fail partway.
    pub fn execute(self, world: &mut World) -> Result<EffectReport, ApprovalError> {
        self.review.validate(world)?;
        let next_revision = world
            .effect_revision
            .checked_add(1)
            .ok_or(ApprovalError::RevisionExhausted)?;
        self.review
            .plan
            .run()
            .map_err(|_| ApprovalError::AlreadyConsumed)?;
        apply_effect(&self.review.plan, world);
        world.effect_revision = next_revision;
        let steps = self.review.plan.steps();
        Ok(EffectReport {
            succeeded: steps
                .iter()
                .filter(|s| s.state == StepState::Succeeded)
                .count(),
            failed: steps
                .iter()
                .filter(|s| matches!(s.state, StepState::Failed(_)))
                .count(),
            skipped: steps
                .iter()
                .filter(|s| matches!(s.state, StepState::Skipped(_) | StepState::PolicySkipped(_)))
                .count(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectReport {
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Failure codes never contain typed confirmation text or target payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalError {
    WrongPhrase,
    StaleTarget,
    AlreadyConsumed,
    RevisionExhausted,
}

impl std::fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongPhrase => "confirmation does not match the reviewed target",
            Self::StaleTarget => "target changed · review a new plan",
            Self::AlreadyConsumed => "plan already ran",
            Self::RevisionExhausted => "simulation revision exhausted",
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

/// Apply the honest effect of a finished plan to the world.
fn apply_effect(plan: &Plan, w: &mut World) {
    if !plan.ran() {
        return;
    }
    let succeeded = |id: &str| {
        plan.steps()
            .iter()
            .find(|s| s.id == id)
            .is_some_and(|s| matches!(s.state, StepState::Succeeded))
    };
    match &plan.effect() {
        Some(PlanEffect::RestartContainer(name)) => {
            if succeeded("restart")
                && let Some(d) = &mut w.docker
                && let Some(c) = d.containers.iter_mut().find(|c| &c.name == name)
            {
                c.state = ContainerState::Running;
                c.health = Some(Health::Healthy);
            }
        }
        Some(PlanEffect::DockerCleanup) => {
            let Some(d) = &mut w.docker else { return };
            if succeeded("cache") {
                d.build_cache_bytes = 0;
            }
            // the failed removal still freed what its output said it removed
            let removed: Vec<String> = if succeeded("containers") {
                d.containers
                    .iter()
                    .filter(|c| c.state == ContainerState::Exited)
                    .map(|c| c.name.clone())
                    .collect()
            } else if plan
                .steps()
                .iter()
                .any(|s| s.id == "containers" && matches!(s.state, StepState::Failed(_)))
            {
                vec!["web".into(), "cron".into()]
            } else {
                vec![]
            };
            d.containers.retain(|c| !removed.contains(&c.name));
            if succeeded("images") {
                d.image_bytes /= 2;
            }
            if succeeded("volumes") {
                d.volume_bytes = 0;
                d.volumes = 0;
            }
        }
        Some(PlanEffect::DiskReclaim) => {
            let Some(disk) = &mut w.disk else { return };
            let freed: Vec<String> = plan
                .steps()
                .iter()
                .filter(|s| s.id.starts_with("rm:") && matches!(s.state, StepState::Succeeded))
                .map(|s| s.id[3..].to_owned())
                .collect();
            let mut bytes = 0u64;
            disk.candidates.retain(|c| {
                if freed.contains(&c.path) {
                    bytes += c.size_bytes;
                    false
                } else {
                    true
                }
            });
            disk.used_bytes = disk.used_bytes.saturating_sub(bytes);
        }
        Some(PlanEffect::GitSync) => {
            let Some(git) = &mut w.git else { return };
            for child in &mut git.children {
                // only a succeeded pull moves a child's counters
                if succeeded(&format!("pull:{}", child.root)) {
                    child.behind = 0;
                }
            }
        }
        Some(PlanEffect::DebianUpgraded) => {
            if succeeded("apt-upgrade")
                && let Some(deb) = &mut w.debian
            {
                deb.pending = deb.held; // held-back packages stay held
                deb.security = 0;
            }
            if succeeded("mise-upgrade")
                && let Some(mise) = &mut w.mise
            {
                for t in &mut mise.tools {
                    if let ToolState::Outdated { latest } = &t.state {
                        t.version = latest.clone();
                        t.state = ToolState::Active;
                    }
                }
            }
        }
        None => {}
    }
}

/// Complete Docker cleanup: inspection, the containers → images/volumes
/// chain, builder cache and networks as parallel branches, verification.
fn docker_cleanup(w: &World) -> Option<Plan> {
    let d = w.docker.as_ref()?;
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
        containers = containers.fails_with(
            "payments-old: bind mount still registered · removal refused",
            &[
                "Removed web",
                "Removed cron",
                "Error: container payments-old: bind mount still registered",
            ],
        );
    } else {
        containers = containers.lines(&[
            &format!("Removed {}", exited.join(", ")),
            &format!("Total reclaimed: {}", human_bytes(d.container_bytes)),
        ]);
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
                d.images,
                human_bytes(d.image_bytes)
            ),
            &format!(
                "Containers          {:<7} {}",
                d.containers.len(),
                human_bytes(d.container_bytes)
            ),
            &format!(
                "Local Volumes       {:<7} {}",
                d.volumes,
                human_bytes(d.volume_bytes)
            ),
            &format!(
                "Build Cache                 {}",
                human_bytes(d.build_cache_bytes)
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
        .lines(&[
            &format!("Deleted {} images", d.images),
            &format!("Total reclaimed: {}", human_bytes(d.image_bytes / 2)),
        ]),
        PlanStep::new(
            "volumes",
            "Remove unused volumes",
            "docker volume prune -f",
            "volumes",
            &[1],
        )
        .lines(&[
            &format!("Deleted {} volumes", d.volumes),
            &format!("Total reclaimed: {}", human_bytes(d.volume_bytes)),
        ]),
        PlanStep::new(
            "cache",
            "Prune builder cache",
            "docker builder prune -a -f",
            "builder",
            &[0],
        )
        .lines(&[
            "Deleted build cache entries",
            &format!("Total reclaimed: {}", human_bytes(d.build_cache_bytes)),
        ]),
        PlanStep::new(
            "networks",
            "Remove unused networks",
            "docker network prune -f",
            "networks",
            &[0],
        )
        .lines(&["Removed 2 unused networks"]),
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
            human_bytes(d.reclaimable_bytes())
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
        .lines(&[
            &format!("Removed {}", c.path),
            &format!("Reclaimed {}", human_bytes(c.size_bytes)),
        ]);
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
        .lines(&[
            &format!(
                "{} upgraded, {} security, {} held back",
                deb.pending - deb.held,
                deb.security,
                deb.held
            ),
            "held back: linux-image-amd64 (kept)",
        ]),
        PlanStep::new(
            "autoremove",
            "Remove orphaned packages",
            "apt autoremove -y",
            "apt",
            &[2],
        )
        .lines(&["Removing 12 orphaned packages", "Freed 410 MB"]),
        PlanStep::new(
            "mise-inspect",
            "Inspect tool versions",
            "mise ls",
            "mise",
            &[0],
        )
        .required()
        .lines(&[
            "node    22.7.0  latest 22.9.0",
            "python  3.12.6  current",
            "rust    1.79.0  latest 1.81.0",
            "go      1.23.0  missing (not required)",
        ]),
        PlanStep::new(
            "mise-upgrade",
            "Upgrade outdated tools",
            "mise up node@22.9.0 rust@1.81.0",
            "mise",
            &[4],
        )
        .lines(&[
            "node 22.7.0 → 22.9.0",
            "rust 1.79.0 → 1.81.0",
            "shims reshimmed",
        ]),
        PlanStep::new(
            "verify",
            "Verify the host",
            "apt list --upgradable && mise ls",
            "verify",
            &[2, 5],
        )
        .required()
        .lines(&[
            "0 upgrades pending · 1 held back (kernel, kept)",
            "all tools current",
            "reboot still required · schedule separately",
        ]),
    ];
    Plan::try_new(PlanSpec {
        action_id: "debian.upgrade".into(),
        title: "Upgrade everything on this host".into(),
        phrase: format!("I UNDERSTAND: UPGRADE EVERYTHING ON {host}"),
        will_change: format!(
            "{} upgrades ({} security) + {} · reboot required after",
            deb.pending,
            deb.security,
            outdated.join(", ")
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
            .lines(&[&format!("Switched to branch '{}'", c.primary_branch)]),
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
            pull.lines(&[
                &format!("Updating origin/{}..", c.primary_branch),
                &format!("Fast-forward · {} commits", c.behind),
            ])
        };
        steps.push(pull);
    }
    let behind_total: u32 = git.children.iter().map(|c| c.behind).sum();
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
        .lines(&[name]),
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
        assert_eq!(d.build_cache_bytes, 0);
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
}
