//! Plan builders: turn a Broad catalogue action into a reviewable DAG of
//! fixture-deterministic steps, and apply the honest world mutation after a
//! run. Phrases are bound to the target host (`REMOVE ALL DOCKER DATA ON
//! devbox`); the broadest plans demand the `I UNDERSTAND:` prefix.

use crate::domain::disk::Freshness;
use crate::domain::docker::{ContainerState, Health};
use crate::domain::human_bytes;
use crate::domain::mise::ToolState;
use crate::domain::plan::{Plan, PlanEffect, PlanStep, StepState};
use crate::sim::world::World;

/// The plan behind a Broad action, if this world has one.
pub fn plan_for(w: &World, action_id: &str) -> Option<Plan> {
    match action_id {
        "docker.cleanup" => docker_cleanup(w),
        "disk.reclaim" => disk_reclaim(w),
        "debian.upgrade" => debian_upgrade(w),
        "git.sync" => git_sync(w),
        id if id.starts_with("docker.restart:") => restart_service(w, &id[15..]),
        _ => None,
    }
}

/// Apply the honest effect of a finished plan to the world.
pub fn apply_effect(plan: &Plan, w: &mut World) {
    if !plan.ran {
        return;
    }
    let succeeded = |id: &str| {
        plan.steps
            .iter()
            .find(|s| s.id == id)
            .is_some_and(|s| matches!(s.state, StepState::Succeeded))
    };
    match &plan.effect {
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
                .steps
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
                .steps
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
    Some(Plan {
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
        ran: false,
    })
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
    Some(Plan {
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
        ran: false,
    })
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
    Some(Plan {
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
        ran: false,
    })
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
    Some(Plan {
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
        ran: false,
    })
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
    Some(Plan {
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
        ran: false,
    })
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
        assert_eq!(p.phrase, "REMOVE ALL DOCKER DATA ON devbox");
        p.run();
        assert!(matches!(p.steps[1].state, StepState::Failed(_)));
        assert!(matches!(p.steps[2].state, StepState::Skipped(_)));
        assert!(matches!(p.steps[4].state, StepState::Succeeded));
        apply_effect(&p, &mut w);
        let d = w.docker.unwrap();
        assert_eq!(d.build_cache_bytes, 0);
        assert!(d.containers.iter().any(|c| c.name == "payments-old"));
        assert!(!d.containers.iter().any(|c| c.name == "web"));
    }

    #[test]
    fn debian_plan_has_parallel_branches_and_policy_phrase() {
        let mut w = settled(Scenario::UpgradePlan);
        let mut p = plan_for(&w, "debian.upgrade").unwrap();
        assert_eq!(p.phrase, "I UNDERSTAND: UPGRADE EVERYTHING ON devbox-deb");
        p.toggle(2).unwrap(); // exclude Apply upgrades
        assert!(p.blocked_by_exclusion(3).is_some(), "autoremove blocked");
        assert!(p.blocked_by_exclusion(6).is_some(), "verify blocked");
        assert_eq!(p.blocked_by_exclusion(5), None, "mise branch parallel");
        p.run();
        assert!(matches!(p.steps[5].state, StepState::Succeeded));
        apply_effect(&p, &mut w);
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
        assert_eq!(p.phrase, "RESTART PAYMENTS ON prod-eu-1");
        p.run();
        apply_effect(&p, &mut w);
        let d = w.docker.as_ref().unwrap();
        assert!(d.unhealthy().is_empty());
    }

    #[test]
    fn disk_plan_policy_skips_active_today() {
        let w = settled(Scenario::DiskCleanup);
        let p = plan_for(&w, "disk.reclaim").unwrap();
        assert_eq!(p.phrase, "DELETE GENERATED ARTIFACTS ON devbox");
        let active = p
            .steps
            .iter()
            .find(|s| matches!(s.state, StepState::PolicySkipped(_)))
            .unwrap();
        assert!(active.title.contains("node_modules"));
    }
}
