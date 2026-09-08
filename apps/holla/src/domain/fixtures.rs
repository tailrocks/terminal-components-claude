//! One coherent fixture world per scenario. Every fact the concept requires
//! (reasons, sizes, freshness, trust, blockers) is seeded here; nothing is
//! probed from the real machine.

use crate::domain::activity::{Activity, ActivityState};
use crate::domain::debian::{DebianState, OrphanPackage};
use crate::domain::disk::{Candidate, DiskState, Family, Freshness};
use crate::domain::docker::{Container, ContainerState, DockerFixture, Health};
use crate::domain::git::{GitRepo, Worktree};
use crate::domain::github::{GhRepo, GhState};
use crate::domain::host::{Environment, Host};
use crate::domain::mise::{MiseState, MiseTask, MiseTool, ToolState, Trust};
use crate::domain::pg::PgSession;
use crate::domain::ranking::{Alias, Memory, Pin, Usage};
use crate::domain::ssh::{HostKeyPolicy, SshHost};
use crate::scenario::Scenario;
use crate::sim::world::{Discovery, Domain, World};

/// Fixture home directory. Display paths use `~`; typed confirmation
/// phrases (P3) expand through this, e.g. `~/work/scratch` →
/// `/home/dev/work/scratch`.
#[allow(dead_code)] // P3 typed-phrase targets
pub(crate) const HOME: &str = "/home/dev";

#[allow(dead_code)] // P3 typed-phrase targets
pub(crate) fn expand_home(path: &str) -> String {
    path.replacen('~', HOME, 1)
}

/// The working directory holla was launched from, per scenario.
pub(crate) fn cwd_for(scenario: Scenario) -> &'static str {
    match scenario {
        Scenario::FirstUse => "~/scratch/empty",
        Scenario::RustDirty => "~/work/pave",
        Scenario::MonorepoRoot => "~/work/monorepo",
        Scenario::MonorepoChild => "~/work/monorepo/apps/frontend",
        Scenario::DockerCleanup => "~/work/scratch",
        Scenario::DiskCleanup => "~/work",
        Scenario::UpgradePlan => "~",
        Scenario::ActivitiesMulti => "~/work/monorepo",
        Scenario::RemoteHost => "/srv/payments",
        Scenario::LaunchFailure => "~/work/pave",
        Scenario::HardCases => "~/work/supercalifragilistic-expiadocious-monorepository-of-doom",
    }
}

pub(crate) fn host_for(scenario: Scenario) -> Host {
    match scenario {
        Scenario::RemoteHost => Host::remote("prod-eu-1", Environment::Production),
        Scenario::UpgradePlan => Host::remote("devbox-deb", Environment::Dev),
        _ => Host::local("devbox"),
    }
}

pub(crate) fn world_for(scenario: Scenario) -> World {
    let mut w = World::new(scenario, host_for(scenario), cwd_for(scenario));
    w.discovery = match scenario {
        // hard-cases: docker discovery never lands; the UI must say so
        Scenario::HardCases => World::default_schedule()
            .into_iter()
            .map(|d| {
                if d.domain == Domain::Docker {
                    Discovery::failing(Domain::Docker, d.at_ms)
                } else {
                    d
                }
            })
            .collect(),
        _ => World::default_schedule(),
    };
    match scenario {
        Scenario::FirstUse => {}
        Scenario::RustDirty => seed_rust_dirty(&mut w),
        Scenario::LaunchFailure => {
            seed_rust_dirty(&mut w);
            // the failure is visible from the first frame: a failed activity
            // in the strip, exit code and all — honesty, not silence
            let mut seed = Activity::new(
                1,
                "seed db",
                "~/work/pave",
                ActivityState::Failed,
                -3_600_000,
            );
            seed.lines = vec![
                "psql: connection to server at \"db.internal\" failed".into(),
                "exit code 1".into(),
            ];
            w.activities = vec![seed];
        }
        Scenario::MonorepoRoot => seed_monorepo(&mut w, false),
        Scenario::MonorepoChild => seed_monorepo(&mut w, true),
        Scenario::DockerCleanup => seed_docker_cleanup(&mut w),
        Scenario::DiskCleanup => seed_disk_cleanup(&mut w),
        Scenario::UpgradePlan => seed_upgrade_plan(&mut w),
        Scenario::ActivitiesMulti => seed_activities_multi(&mut w),
        Scenario::RemoteHost => seed_remote_host(&mut w),
        Scenario::HardCases => seed_hard_cases(&mut w),
    }
    w
}

// ---------------------------------------------------------------- helpers

fn tool(name: &str, version: &str, state: ToolState) -> MiseTool {
    MiseTool {
        name: name.into(),
        version: version.into(),
        state,
    }
}

fn task(id: &str, command: &str, defined_in: &str) -> MiseTask {
    MiseTask {
        id: id.into(),
        command: command.into(),
        defined_in: defined_in.into(),
        trust: Trust::Trusted,
    }
}

fn container(name: &str, image: &str, state: ContainerState, health: Option<Health>) -> Container {
    Container {
        size_bytes: 0,
        name: name.into(),
        image: image.into(),
        state,
        health,
    }
}

fn candidate(path: &str, family: Family, size_bytes: u64, freshness: Freshness) -> Candidate {
    Candidate {
        path: path.into(),
        family,
        size_bytes,
        freshness,
    }
}

/// A nested non-submodule child repo. `branch: None` = detached at `sha`;
/// `primary` is resolved per child, never assumed.
fn child_repo(
    root: &str,
    branch: Option<&str>,
    sha: Option<&str>,
    primary: &str,
    ahead: u32,
    behind: u32,
) -> GitRepo {
    GitRepo {
        root: root.into(),
        branch: branch.map(str::to_owned),
        detached_sha: sha.map(str::to_owned),
        upstream: branch.map(|b| format!("origin/{b}")),
        ahead,
        behind,
        modified: 0,
        staged: 0,
        untracked: 0,
        primary_branch: primary.into(),
        worktrees: vec![],
        submodules: vec![],
        children: vec![],
    }
}

const GB: u64 = 1_073_741_824;
const MB: u64 = 1_048_576;

fn ssh_hosts_devbox() -> Vec<SshHost> {
    vec![
        SshHost {
            alias: "devbox-deb".into(),
            host_name: "192.168.64.9".into(),
            user: "dev".into(),
            port: None,
            identity_file: Some("id_ed25519".into()),
            jump: None,
            host_key_policy: HostKeyPolicy::Strict,
            multiplexed: true,
        },
        SshHost {
            alias: "prod-eu-1".into(),
            host_name: "10.20.30.40".into(),
            user: "deploy".into(),
            port: None,
            identity_file: Some("id_ed25519_prod".into()),
            jump: Some("bastion".into()),
            host_key_policy: HostKeyPolicy::Strict,
            multiplexed: false,
        },
        SshHost {
            alias: "bastion".into(),
            host_name: "bastion.example.com".into(),
            user: "ops".into(),
            port: Some(2222),
            identity_file: Some("id_ed25519".into()),
            jump: None,
            host_key_policy: HostKeyPolicy::Strict,
            multiplexed: true,
        },
    ]
}

// -------------------------------------------------------------- scenarios

/// ~/work/pave: Rust project, dirty worktree, branch behind upstream.
fn seed_rust_dirty(w: &mut World) {
    w.mise = Some(MiseState {
        tools: vec![
            tool("rust", "1.81.0", ToolState::Active),
            tool(
                "node",
                "20.11.0",
                ToolState::Outdated {
                    latest: "22.9.0".into(),
                },
            ),
            tool("just", "1.36.0", ToolState::Missing),
        ],
        tasks: vec![
            task("test", "cargo test --workspace", "~/work/pave/mise.toml"),
            task("build", "cargo build --workspace", "~/work/pave/mise.toml"),
            task(
                "lint",
                "cargo clippy --all-targets",
                "~/work/pave/mise.toml",
            ),
        ],
    });
    w.git = Some(GitRepo {
        root: "~/work/pave".into(),
        branch: Some("feature/cache".into()),
        detached_sha: None,
        upstream: Some("origin/feature/cache".into()),
        ahead: 0,
        behind: 3,
        modified: 4,
        staged: 1,
        untracked: 2,
        primary_branch: "trunk".into(),
        worktrees: vec![Worktree {
            path: "~/work/pave-hotfix".into(),
            branch: "hotfix/404".into(),
        }],
        submodules: vec!["vendor/micropkg".into()],
        children: vec![],
    });
    w.ssh = ssh_hosts_devbox();
    w.github = Some(GhState {
        login: "octocat".into(),
        orgs: vec!["pave-io".into()],
        repos: vec![
            GhRepo {
                owner: "pave-io".into(),
                name: "pave".into(),
                default_branch: "trunk".into(),
                private: true,
            },
            GhRepo {
                owner: "octocat".into(),
                name: "dotfiles".into(),
                default_branch: "main".into(),
                private: false,
            },
        ],
    });
    w.disk = Some(DiskState {
        total_bytes: 500 * GB,
        used_bytes: 240 * GB,
        candidates: vec![candidate(
            "~/work/pave/target",
            Family::CargoTarget,
            900 * MB,
            Freshness::ActiveToday,
        )],
    });
    w.memory = Memory::seeded(
        vec![Pin {
            path: "~/work/pave".into(),
            command: "make test".into(),
        }],
        vec![Alias {
            alias: "gs".into(),
            expansion: "git status".into(),
        }],
        vec![
            Usage {
                path: "~/work/pave".into(),
                command: "cargo build".into(),
                count: 6,
            },
            Usage {
                path: "~/work/pave".into(),
                command: "cargo test".into(),
                count: 4,
            },
        ],
        vec![],
    );
}

/// ~/work/monorepo (root or apps/frontend child): namespaced mise tasks,
/// one untrusted task file, a nested non-submodule child repo.
fn seed_monorepo(w: &mut World, child: bool) {
    let mut tasks = vec![
        task(
            "//:lint",
            "eslint . --max-warnings 0",
            "~/work/monorepo/mise.toml",
        ),
        task(
            "//projects/frontend:build",
            "pnpm --filter frontend build",
            "~/work/monorepo/projects/frontend/mise.toml",
        ),
        task(
            "//projects/frontend:dev",
            "pnpm --filter frontend dev",
            "~/work/monorepo/projects/frontend/mise.toml",
        ),
        task(
            "//projects/backend:test",
            "cargo test -p backend",
            "~/work/monorepo/projects/backend/mise.toml",
        ),
    ];
    let mut reset_db = task(
        "//projects/backend:reset-db",
        "psql -c 'DROP SCHEMA public CASCADE'",
        "~/work/monorepo/projects/backend/mise.toml",
    );
    reset_db.trust = Trust::Untrusted;
    tasks.push(reset_db);
    if child {
        tasks.push(task(
            "dev",
            "vite --port 5199",
            "~/work/monorepo/apps/frontend/mise.toml",
        ));
    }
    w.mise = Some(MiseState {
        tools: vec![
            tool("node", "22.9.0", ToolState::Active),
            tool("pnpm", "9.12.0", ToolState::Active),
            tool("rust", "1.81.0", ToolState::Active),
        ],
        tasks,
    });
    w.git = Some(GitRepo {
        root: "~/work/monorepo".into(),
        branch: Some("trunk".into()),
        detached_sha: None,
        upstream: Some("origin/trunk".into()),
        ahead: 0,
        behind: 0,
        modified: 0,
        staged: 0,
        untracked: 0,
        primary_branch: "trunk".into(),
        worktrees: vec![],
        submodules: vec![],
        children: vec![
            child_repo(
                "~/work/monorepo/services/legacy",
                Some("main"),
                None,
                "main",
                0,
                5,
            ),
            child_repo(
                "~/work/monorepo/services/billing",
                Some("trunk"),
                None,
                "trunk",
                2,
                3,
            ),
            child_repo(
                "~/work/monorepo/tools/seed",
                None,
                Some("a1b2c3d"),
                "main",
                0,
                0,
            ),
        ],
    });
    w.docker = Some(
        DockerFixture {
            containers: vec![
                container(
                    "postgres",
                    "postgres:16",
                    ContainerState::Running,
                    Some(Health::Healthy),
                ),
                container("redis", "redis:7", ContainerState::Running, None),
                container("mailhog", "mailhog/mailhog", ContainerState::Exited, None),
            ],
            images: 6,
            networks: 4,
            volumes: 3,
            image_bytes: 3_800 * MB,
            container_bytes: 120 * MB,
            volume_bytes: 1_200 * MB,
            build_cache_bytes: 2_100 * MB,
        }
        .into_state(),
    );
    w.disk = Some(DiskState {
        total_bytes: 500 * GB,
        used_bytes: 300 * GB,
        candidates: vec![candidate(
            "~/work/monorepo/apps/frontend/node_modules",
            Family::NodeModules,
            3_100 * MB,
            Freshness::ActiveToday,
        )],
    });
    w.github = Some(GhState {
        login: "octocat".into(),
        orgs: vec!["acme".into()],
        repos: vec![GhRepo {
            owner: "acme".into(),
            name: "monorepo".into(),
            default_branch: "main".into(),
            private: true,
        }],
    });
    w.memory = Memory::seeded(
        vec![],
        vec![],
        vec![if child {
            Usage {
                path: "~/work/monorepo/apps/frontend".into(),
                command: "pnpm dev".into(),
                count: 9,
            }
        } else {
            Usage {
                path: "~/work/monorepo".into(),
                command: "mise run //projects/frontend:build".into(),
                count: 3,
            }
        }],
        vec![],
    );
}

/// ~/work/scratch: a Docker host whose complete cleanup is the point.
fn seed_docker_cleanup(w: &mut World) {
    w.docker = Some(
        DockerFixture {
            containers: vec![
                container(
                    "api",
                    "acme/api:1.4",
                    ContainerState::Running,
                    Some(Health::Healthy),
                ),
                container(
                    "worker",
                    "acme/api:1.4",
                    ContainerState::Running,
                    Some(Health::Healthy),
                ),
                container("redis", "redis:7", ContainerState::Running, None),
                container(
                    "legacy-billing",
                    "acme/billing:0.9",
                    ContainerState::Restarting,
                    Some(Health::Unhealthy),
                ),
                container("web", "acme/web:2.0", ContainerState::Exited, None),
                container("payments-old", "acme/pay:0.3", ContainerState::Exited, None),
                container("cron", "acme/cron:1.1", ContainerState::Exited, None),
            ],
            images: 9,
            networks: 6,
            volumes: 5,
            image_bytes: 6_200 * MB,
            container_bytes: 420 * MB,
            volume_bytes: 3_400 * MB,
            build_cache_bytes: 12_700 * MB,
        }
        .into_state(),
    );
    w.disk = Some(DiskState {
        total_bytes: 250 * GB,
        used_bytes: 210 * GB,
        candidates: vec![
            candidate(
                "/var/lib/docker",
                Family::DockerData,
                23 * GB,
                Freshness::ActiveToday,
            ),
            candidate(
                "~/work/scratch/tmp",
                Family::Temp,
                300 * MB,
                Freshness::Unknown,
            ),
        ],
    });
    w.memory = Memory::seeded(
        vec![],
        vec![],
        vec![Usage {
            path: "~/work/scratch".into(),
            command: "docker system df".into(),
            count: 4,
        }],
        vec![],
    );
}

/// ~/work: disk at 92%, progressive candidates with freshness facts.
fn seed_disk_cleanup(w: &mut World) {
    w.disk = Some(DiskState {
        total_bytes: 500 * GB,
        used_bytes: 460 * GB,
        candidates: vec![
            candidate(
                "~/work/forge/target",
                Family::CargoTarget,
                8_400 * MB,
                Freshness::InactiveDays(31),
            ),
            candidate(
                "~/work/monorepo/apps/frontend/node_modules",
                Family::NodeModules,
                3_100 * MB,
                Freshness::ActiveToday,
            ),
            candidate(
                "~/work/shop/.gradle",
                Family::Gradle,
                12_700 * MB,
                Freshness::Unknown,
            ),
            candidate(
                "~/.cache/mise",
                Family::PackageCache,
                900 * MB,
                Freshness::InactiveDays(14),
            ),
            candidate("/var/log/old", Family::Logs, 640 * MB, Freshness::Unknown),
            candidate(
                "~/tmp/scratch",
                Family::Temp,
                300 * MB,
                Freshness::InactiveDays(90),
            ),
        ],
    });
}

/// devbox-deb: Debian host, global mise tools, the upgrade-everything plan.
fn seed_upgrade_plan(w: &mut World) {
    w.debian = Some(DebianState {
        orphaned_packages: (0..12)
            .map(|index| OrphanPackage {
                id: format!("fixture:orphan:{index}"),
                size_bytes: 410 * MB / 12 + u64::from(index < 410 * MB % 12),
            })
            .collect(),
        pending: 47,
        security: 6,
        held: 1,
        reboot_required: true,
    });
    w.mise = Some(MiseState {
        tools: vec![
            tool(
                "node",
                "22.7.0",
                ToolState::Outdated {
                    latest: "22.9.0".into(),
                },
            ),
            tool("python", "3.12.6", ToolState::Active),
            tool(
                "rust",
                "1.79.0",
                ToolState::Outdated {
                    latest: "1.81.0".into(),
                },
            ),
            tool("go", "1.23.0", ToolState::Missing),
        ],
        tasks: vec![],
    });
    w.docker = Some(
        DockerFixture {
            containers: vec![container(
                "gitea",
                "gitea/gitea:1.22",
                ContainerState::Running,
                Some(Health::Healthy),
            )],
            images: 3,
            networks: 2,
            volumes: 2,
            image_bytes: 1_100 * MB,
            container_bytes: 60 * MB,
            volume_bytes: 800 * MB,
            build_cache_bytes: 2_200 * MB,
        }
        .into_state(),
    );
    w.disk = Some(DiskState {
        total_bytes: 100 * GB,
        used_bytes: 71 * GB,
        candidates: vec![candidate(
            "~/.cache/mise",
            Family::PackageCache,
            900 * MB,
            Freshness::InactiveDays(21),
        )],
    });
    w.ssh = vec![SshHost {
        alias: "bastion".into(),
        host_name: "bastion.example.com".into(),
        user: "ops".into(),
        port: Some(2222),
        identity_file: Some("id_ed25519".into()),
        jump: None,
        host_key_policy: HostKeyPolicy::Strict,
        multiplexed: true,
    }];
}

/// ~/work/monorepo with four named activities in distinct states.
fn seed_activities_multi(w: &mut World) {
    seed_monorepo(w, false);
    let hour = 3_600_000;
    let mut dev = Activity::new(
        1,
        "dev server",
        "~/work/monorepo/apps/frontend",
        ActivityState::Running,
        -2 * hour,
    );
    dev.lines = vec![
        "vite v5.4 ready in 412 ms".into(),
        "➜ Local: http://localhost:5199/".into(),
    ];
    let mut watch = Activity::new(
        2,
        "test watch",
        "~/work/monorepo",
        ActivityState::Waiting,
        -600_000,
    );
    watch.lines = vec!["waiting for file changes…".into()];
    let mut deploy = Activity::new(
        3,
        "deploy logs",
        "~/work/monorepo/services/api",
        ActivityState::Detached,
        -24 * hour,
    );
    deploy.lines = vec!["deploy 2026-09-05T09:12: finished".into()];
    let mut seed = Activity::new(
        4,
        "seed db",
        "~/work/monorepo",
        ActivityState::Failed,
        -hour,
    );
    seed.lines = vec![
        "psql: connection to server at \"db.internal\" failed".into(),
        "exit code 1".into(),
    ];
    w.activities = vec![dev, watch, deploy, seed];
}

/// /srv/payments on ◆ prod-eu-1: sensitive identity, unhealthy service,
/// a pg blocker tree.
fn seed_remote_host(w: &mut World) {
    w.git = Some(GitRepo {
        root: "/srv/payments".into(),
        branch: Some("release/2026.08".into()),
        detached_sha: None,
        upstream: Some("origin/release/2026.08".into()),
        ahead: 0,
        behind: 1,
        modified: 0,
        staged: 0,
        untracked: 0,
        primary_branch: "main".into(),
        worktrees: vec![],
        submodules: vec![],
        children: vec![],
    });
    w.docker = Some(
        DockerFixture {
            containers: vec![
                container(
                    "payments",
                    "acme/payments:2026.08.3",
                    ContainerState::Restarting,
                    Some(Health::Unhealthy),
                ),
                container(
                    "postgres",
                    "postgres:16",
                    ContainerState::Running,
                    Some(Health::Healthy),
                ),
                container("nginx", "nginx:1.27", ContainerState::Running, None),
            ],
            images: 12,
            networks: 5,
            volumes: 8,
            image_bytes: 9_800 * MB,
            container_bytes: 900 * MB,
            volume_bytes: 14_000 * MB,
            build_cache_bytes: 4_500 * MB,
        }
        .into_state(),
    );
    w.pg = Some(vec![
        PgSession {
            pid: 4201,
            user: "payments".into(),
            query: "ALTER TABLE payments ADD COLUMN …".into(),
            wait_event: None,
            blocked_by: None,
            duration_ms: 47 * 60_000,
        },
        PgSession {
            pid: 4217,
            user: "app_ro".into(),
            query: "SELECT … FROM payments".into(),
            wait_event: Some("Lock:relation".into()),
            blocked_by: Some(4201),
            duration_ms: 41 * 60_000,
        },
        PgSession {
            pid: 4223,
            user: "report".into(),
            query: "SELECT count(*) FROM …".into(),
            wait_event: Some("Lock:relation".into()),
            blocked_by: Some(4201),
            duration_ms: 12 * 60_000,
        },
        PgSession {
            pid: 4300,
            user: "health".into(),
            query: "SELECT 1".into(),
            wait_event: None,
            blocked_by: None,
            duration_ms: 300,
        },
    ]);
    w.debian = Some(DebianState {
        orphaned_packages: Vec::new(),
        pending: 12,
        security: 3,
        held: 0,
        reboot_required: false,
    });
    w.ssh = vec![SshHost {
        alias: "bastion".into(),
        host_name: "bastion.example.com".into(),
        user: "ops".into(),
        port: Some(2222),
        identity_file: None,
        jump: None,
        host_key_policy: HostKeyPolicy::Strict,
        multiplexed: false,
    }];
}

/// Long labels, missing data, detached HEAD, a docker discovery failure.
fn seed_hard_cases(w: &mut World) {
    w.mise = Some(MiseState {
        tools: vec![],
        tasks: vec![task(
            "//projects/extremely-long-service-name-that-goes-on:very-long-task-name",
            "true",
            "~/work/supercalifragilistic-expiadocious-monorepository-of-doom/mise.toml",
        )],
    });
    w.git = Some(GitRepo {
        root: "~/work/supercalifragilistic-expiadocious-monorepository-of-doom".into(),
        branch: None,
        detached_sha: Some("9f3a21c".into()),
        upstream: None,
        ahead: 0,
        behind: 0,
        modified: 0,
        staged: 0,
        untracked: 0,
        primary_branch: "trunk".into(),
        worktrees: vec![],
        submodules: vec![],
        children: vec![child_repo(
            "~/work/supercalifragilistic-expiadocious-monorepository-of-doom/services/extremely-long-nested-child-repository",
            None,
            Some("7e8f9a0"),
            "trunk",
            0,
            0,
        )],
    });
    w.disk = Some(DiskState {
        total_bytes: 500 * GB,
        used_bytes: 499 * GB,
        candidates: vec![candidate(
            "~/work/supercalifragilistic-expiadocious-monorepository-of-doom/services/extremely-long-service-name/generated-artifacts",
            Family::DistBuild,
            12_700 * MB,
            Freshness::Unknown,
        )],
    });
    w.ssh = vec![SshHost {
        alias: "legacy-2009".into(),
        host_name: "10.255.255.1".into(),
        user: "root".into(),
        port: None,
        identity_file: None,
        jump: None,
        host_key_policy: HostKeyPolicy::Ask,
        multiplexed: false,
    }];
}

/// Explicit authorization for simulated custom commands seeded by these
/// fixtures. Stored text alone never grants an effect or a risk downgrade.
pub(crate) fn memory_command(
    scenario: Scenario,
    cwd: &str,
    command: &str,
) -> Option<crate::domain::action::FixtureCommand> {
    use crate::domain::action::FixtureCommand;
    let allowed: &[FixtureCommand] = match (scenario, cwd) {
        (Scenario::RustDirty | Scenario::LaunchFailure, "~/work/pave") => &[
            FixtureCommand::MakeTest,
            FixtureCommand::CargoBuild,
            FixtureCommand::CargoTest,
        ],
        (Scenario::MonorepoChild, "~/work/monorepo/apps/frontend") => &[FixtureCommand::PnpmDev],
        (Scenario::DockerCleanup, "~/work/scratch") => &[FixtureCommand::DockerUsage],
        _ => &[],
    };
    allowed
        .iter()
        .copied()
        .find(|candidate| candidate.command() == command)
}

#[cfg(test)]
#[expect(clippy::indexing_slicing, clippy::unwrap_used, reason = "Tests assert fixed fixture structure and bounded values; violations must fail the test")]
mod tests {
    use super::*;
    use crate::domain::pg;

    fn settled(scenario: Scenario) -> World {
        let mut w = world_for(scenario);
        w.clock.running = true;
        w.seek(10_000);
        w
    }

    #[test]
    fn every_scenario_settles_discovery() {
        for s in Scenario::ALL {
            let w = settled(s);
            assert!(!w.discovering(), "{} still discovering", s.name());
        }
    }

    #[test]
    fn hard_cases_docker_discovery_fails() {
        let w = settled(Scenario::HardCases);
        assert!(w.discovery_failed(Domain::Docker));
        assert!(w.discovered(Domain::Git));
        assert!(w.docker.is_none());
    }

    #[test]
    fn rust_dirty_facts_match_the_vocabulary() {
        let w = settled(Scenario::RustDirty);
        let git = w.git.as_ref().unwrap();
        assert_eq!(git.behind, 3);
        assert_eq!(git.modified, 4);
        assert_eq!(git.primary_branch, "trunk");
        assert!(
            git.summary()
                .contains(&"branch is 3 commits behind".to_owned())
        );
        assert!(git.summary().contains(&"4 modified files".to_owned()));
        assert_eq!(w.memory.usage_at("~/work/pave", "cargo build"), 6);
        assert!(w.memory.pin_at("~/work/pave", "make test"));
        assert_eq!(w.memory.alias("gs"), Some("git status"));
    }

    #[test]
    fn docker_cleanup_has_serious_reclaimable() {
        let w = settled(Scenario::DockerCleanup);
        let d = w.docker.as_ref().unwrap();
        assert_eq!(d.unhealthy().len(), 1);
        assert!(
            d.reclaimable_bytes().unwrap() >= 12 * GB,
            "{}",
            d.reclaimable_bytes().unwrap()
        );
        assert_eq!(d.running(), 3);
    }

    #[test]
    fn remote_host_has_blocker_tree_and_production_identity() {
        let w = settled(Scenario::RemoteHost);
        assert_eq!(w.host.env, Environment::Production);
        let sessions = w.pg.as_ref().unwrap();
        let roots = pg::blockers(sessions);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].pid, 4201);
        assert_eq!(pg::blocked_by(sessions, 4201).len(), 2);
        let d = w.docker.as_ref().unwrap();
        assert_eq!(d.unhealthy()[0].name, "payments");
    }

    #[test]
    fn activities_multi_carries_four_states() {
        let w = settled(Scenario::ActivitiesMulti);
        assert_eq!(w.activities.len(), 4);
        for state in [
            ActivityState::Running,
            ActivityState::Waiting,
            ActivityState::Detached,
            ActivityState::Failed,
        ] {
            assert!(
                w.activities.iter().any(|a| a.state == state),
                "missing {state:?}"
            );
        }
    }

    #[test]
    fn upgrade_plan_facts() {
        let w = settled(Scenario::UpgradePlan);
        let deb = w.debian.as_ref().unwrap();
        assert_eq!(deb.pending, 47);
        assert!(deb.reboot_required);
        let mise = w.mise.as_ref().unwrap();
        assert!(mise.tools.iter().any(|t| t.state == ToolState::Missing));
        assert_eq!(
            mise.tools
                .iter()
                .filter(|t| matches!(t.state, ToolState::Outdated { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn monorepo_child_sees_parent_tasks_and_local_dev() {
        let w = settled(Scenario::MonorepoChild);
        let mise = w.mise.as_ref().unwrap();
        assert!(
            mise.tasks
                .iter()
                .any(|t| t.id == "//projects/frontend:build")
        );
        assert!(mise.tasks.iter().any(|t| t.id == "dev"));
        assert!(
            mise.tasks
                .iter()
                .any(|t| t.trust == Trust::Untrusted && t.id == "//projects/backend:reset-db")
        );
    }

    #[test]
    fn worlds_are_deterministic() {
        let a = settled(Scenario::DiskCleanup);
        let b = settled(Scenario::DiskCleanup);
        assert_eq!(a.disk, b.disk);
        assert_eq!(a.memory, b.memory);
        assert_eq!(a.git, b.git);
    }

    #[test]
    fn expand_home_resolves_typed_phrase_targets() {
        assert_eq!(expand_home("~/work/scratch"), "/home/dev/work/scratch");
    }
}
