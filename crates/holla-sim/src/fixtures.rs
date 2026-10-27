use crate::activity_supervisor::{
    ActivityExecutionStart, ActivityStopProfile, ActivitySupervisorError,
};
use crate::domain::activity::{Activity, ActivityState};
use crate::domain::debian::{DebianState, OrphanPackage};
use crate::domain::disk::{Candidate, DiskState, Family, Freshness};
use crate::domain::docker::{Container, ContainerState, DockerFixture, Health};
use crate::domain::git::{GitRepo, Worktree};
use crate::domain::github::{GhRepo, GhState};
use crate::domain::host::{Environment, Host};
use crate::domain::mise::{MiseState, MiseTask, MiseTool, ToolState, Trust};
use crate::domain::pg::PgSession;
use crate::domain::ranking::{Alias, HostUsage, Memory, Pin, Usage};
use crate::domain::ssh::{HostKeyPolicy, SshHost};
use crate::listeners::{ListenerModel, ListenerModelError, ListenerRecord, PortConflict};
use crate::scenario::Scenario;
use crate::world::{Discovery, Domain, SystemProcess, SystemSnapshot, World};

pub const HOME: &str = "/home/dev";

pub fn expand_home(path: &str) -> String {
    path.replacen('~', HOME, 1)
}

pub fn files_for_cwd(cwd: &str) -> crate::files::FilesSnapshot {
    use crate::files::{FileEntry, FilesSnapshot};

    fn add_directory_chain(snapshot: &mut FilesSnapshot, cwd: &str) {
        if cwd.starts_with("~/") || cwd == "~" {
            snapshot.insert(FileEntry::directory("~"));
            let mut current = "~".to_owned();
            let suffix = if cwd == "~" {
                ""
            } else {
                cwd.trim_start_matches("~/")
            };
            for segment in suffix.split('/').filter(|part| !part.is_empty()) {
                current = FilesSnapshot::join(&current, segment);
                snapshot.insert(FileEntry::directory(current.clone()));
            }
        } else {
            snapshot.insert(FileEntry::directory("/"));
            let mut current = "/".to_owned();
            for segment in cwd
                .trim_start_matches('/')
                .split('/')
                .filter(|part| !part.is_empty())
            {
                current = FilesSnapshot::join(&current, segment);
                snapshot.insert(FileEntry::directory(current.clone()));
            }
        }
    }

    let cwd = if cwd.is_empty() {
        "/"
    } else {
        cwd.trim_end_matches('/')
    };
    let cwd = if cwd.is_empty() { "/" } else { cwd };
    let mut snapshot = FilesSnapshot::new();
    add_directory_chain(&mut snapshot, cwd);
    if cwd.ends_with("/empty") || cwd == "~" || cwd == "/" {
        return snapshot;
    }

    let archive = FilesSnapshot::join(cwd, "archive");
    let drafts = FilesSnapshot::join(cwd, "drafts");
    let private = FilesSnapshot::join(cwd, "restricted");
    snapshot.insert(FileEntry::directory(archive.clone()));
    snapshot.insert(FileEntry::directory(drafts.clone()));
    snapshot.insert(FileEntry::directory(private).unreadable());
    snapshot.insert(FileEntry::file(FilesSnapshot::join(cwd, ".notes-index")));
    snapshot.insert(FileEntry::file_with_bytes(
        FilesSnapshot::join(cwd, "README.md"),
        b"# Simulated workspace\n\nThis file is part of the in-memory fixture.\n".to_vec(),
    ));
    snapshot.insert(FileEntry::file(FilesSnapshot::join(cwd, "résumé.txt")));
    snapshot.insert(FileEntry::symlink(
        FilesSnapshot::join(cwd, "broken-shortcut"),
        FilesSnapshot::join(cwd, "missing-target"),
    ));
    snapshot.insert(FileEntry::file(FilesSnapshot::join(&archive, "2025.txt")));
    snapshot.insert(FileEntry::file_with_bytes(
        FilesSnapshot::join(&drafts, "meeting-notes.md"),
        b"Meeting notes\n\n- Review the preview flow\n".to_vec(),
    ));
    snapshot.insert(FileEntry::file(FilesSnapshot::join(&drafts, "秘密.md")));
    snapshot
}

pub fn cwd_for(scenario: Scenario) -> &'static str {
    match scenario {
        Scenario::ParityActivityStop => "~/work/activity-stop",
        Scenario::FirstUse => "~/scratch",
        Scenario::RustDirty | Scenario::LaunchFailure => "~/work/pave",
        Scenario::MonorepoRoot | Scenario::ActivitiesMulti => "~/work/monorepo",
        Scenario::MonorepoChild => "~/work/monorepo/apps/frontend",
        Scenario::DockerCleanup => "~/work/scratch",
        Scenario::DiskCleanup => "~/work",
        Scenario::UpgradePlan => "~",
        Scenario::RemoteHost | Scenario::ParityTaskInput => "/srv/payments",
        Scenario::HardCases => "~/work/supercalifragilistic-expiadocious-monorepository-of-doom",
        Scenario::ParityDiscovery => "~/work/probe",
        Scenario::ParityHistory => "~/work/holla",
        Scenario::ParityFiles => "~/work/notes",
        Scenario::ParityBrowser => "~/work/site",
        Scenario::ParityGitCurrent => "~/work/svc",
        Scenario::ParityGitBatch => "~/work/repos",
        Scenario::ParityTaskSources => "~/work/poly",
        Scenario::ParityCargo => "~/work/engine",
        Scenario::ParityDocker => "~/work/docker",
        Scenario::ParityBrewServices => "~/work/brew",
        Scenario::ParityGradle => "~/work/android",
        Scenario::ParityIdea => "~/work/idea",
        Scenario::ParityUpgradeManagers => "~/work/upgrades",
        Scenario::ParityExecutor => "~/work/batch",
        Scenario::ParityCustomActions => "~/work/team",
        Scenario::ParityDiskScan => "~/work/data",
        Scenario::ParityDiskNavigation => "~/work/big",
        Scenario::ParityInsights => "~/work/web",
        Scenario::ParityDeleteSafety | Scenario::ParityCleanupResults => "~/Projects/safe",
        Scenario::ParityPlatforms => "~/work/mac",
        Scenario::ParityPlatformsLinux => "/home/alex",
    }
}

pub fn host_for(scenario: Scenario) -> Host {
    match scenario {
        Scenario::RemoteHost => Host::remote("prod-eu-1", Environment::Production),
        Scenario::UpgradePlan => Host::remote("devbox-deb", Environment::Dev),
        Scenario::ParityTaskInput => Host::remote("devbox", Environment::Production),
        Scenario::ParityPlatformsLinux => Host::local("devbox"),
        Scenario::RustDirty
        | Scenario::MonorepoRoot
        | Scenario::MonorepoChild
        | Scenario::DockerCleanup
        | Scenario::DiskCleanup
        | Scenario::ActivitiesMulti
        | Scenario::LaunchFailure
        | Scenario::HardCases => Host::local("devbox"),
        Scenario::ParityActivityStop => Host::local("devbox"),
        Scenario::FirstUse => Host::local("mbp"),
        _ => Host::local("mbp"),
    }
}

pub fn world_for(scenario: Scenario) -> World {
    let mut w = World::new(scenario, host_for(scenario), cwd_for(scenario));
    w.discovery = match scenario {
        Scenario::FirstUse => vec![
            Discovery::at(Domain::Filesystem, 240),
            Discovery::at(Domain::Git, 480),
            Discovery::at(Domain::Mise, 720),
            Discovery::at(Domain::System, 960),
            Discovery::at(Domain::Ssh, 1_280),
            Discovery::at(Domain::Github, 3_200),
        ],
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
        Scenario::FirstUse => seed_first_use(&mut w),
        Scenario::RustDirty => seed_rust_dirty(&mut w),
        Scenario::LaunchFailure => {
            seed_rust_dirty(&mut w);

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
        Scenario::ParityActivityStop => return parity_activity_stop_world(w),
        Scenario::RemoteHost => seed_remote_host(&mut w),
        Scenario::HardCases => seed_hard_cases(&mut w),
        Scenario::ParityDiscovery
        | Scenario::ParityHistory
        | Scenario::ParityFiles
        | Scenario::ParityBrowser
        | Scenario::ParityGitCurrent
        | Scenario::ParityGitBatch
        | Scenario::ParityTaskSources
        | Scenario::ParityCargo
        | Scenario::ParityDocker
        | Scenario::ParityBrewServices
        | Scenario::ParityGradle
        | Scenario::ParityIdea
        | Scenario::ParityUpgradeManagers
        | Scenario::ParityExecutor
        | Scenario::ParityTaskInput
        | Scenario::ParityCustomActions
        | Scenario::ParityDiskScan
        | Scenario::ParityDiskNavigation
        | Scenario::ParityInsights
        | Scenario::ParityDeleteSafety
        | Scenario::ParityCleanupResults
        | Scenario::ParityPlatforms
        | Scenario::ParityPlatformsLinux => {}
    }
    w
}

fn seed_first_use(world: &mut World) {
    world.host.metrics.load_x100 = (940, 780, 510);
    world.host.metrics.mem_used_mb = 12_698;
    world.host.metrics.mem_total_mb = 16_384;
    world.disk = Some(DiskState {
        total_bytes: 926 * GB,
        used_bytes: 842 * GB,
        candidates: Vec::new(),
    });
    world.mise = Some(MiseState {
        tools: vec![tool("node", "26.0.1", ToolState::Active)],
        tasks: Vec::new(),
    });
    world.system = Some(SystemSnapshot {
        cpu_pct: 78,
        cores: vec![91, 84, 70, 66, 88, 79, 71, 75],
        swap_used_mb: 1_126,
        pressure: None,
        pressure_minutes: 4,
        disk_io: vec![("disk0".into(), "14.2 MB/s read · 3.1 MB/s write".into())],
        network: vec![("en0".into(), "rx 1.2 MB/s · tx 240 KB/s".into())],
        processes: vec![
            SystemProcess {
                pid: 4812,
                parent_pid: Some(1),
                name: "rust-analyzer".into(),
                cpu_pct: 41,
                mem_mb: 2_900,
                state: "running".into(),
                port: None,
            },
            SystemProcess {
                pid: 9120,
                parent_pid: Some(4812),
                name: "cargo".into(),
                cpu_pct: 32,
                mem_mb: 1_100,
                state: "running".into(),
                port: None,
            },
            SystemProcess {
                pid: 2201,
                parent_pid: Some(1),
                name: "Docker".into(),
                cpu_pct: 9,
                mem_mb: 3_800,
                state: "sleeping".into(),
                port: None,
            },
            SystemProcess {
                pid: 7731,
                parent_pid: Some(1),
                name: "node (vite)".into(),
                cpu_pct: 6,
                mem_mb: 420,
                state: "sleeping".into(),
                port: Some(5_173),
            },
        ],
        btm_installed: true,
        os: "macOS".into(),
        temp_c: Some(87),
    });
    world.memory.host_usage.push(HostUsage {
        host: "mbp".into(),
        action_id: "system.resources".into(),
        count: 2,
        last_used_ms: -259_200_000,
    });
}

pub fn nextest_args_world() -> World {
    let mut world = world_for(Scenario::RustDirty);
    if let Some(mise) = &mut world.mise {
        mise.tasks.push(task(
            "test-nextest",
            "cargo nextest run",
            "~/work/pave/mise.toml",
        ));
    }
    for discovery in &mut world.discovery {
        if discovery.domain == Domain::Mise {
            discovery.done = true;
        }
    }
    world
}

pub struct PortConflictCase {
    pub world: World,

    pub listeners: Option<ListenerModel>,

    pub conflict: PortConflict,
}

pub fn port_conflict_case() -> Result<PortConflictCase, ListenerModelError> {
    let mut world = world_for(Scenario::MonorepoChild);
    let mut activity = Activity::new(
        1_000_001,
        "frontend dev",
        &world.cwd,
        ActivityState::Failed,
        world.now_ms(),
    );
    activity.lines = vec![
        "npm run dev".to_owned(),
        "Error: listen EADDRINUSE: address already in use 127.0.0.1:5173".to_owned(),
        "exit code 1".to_owned(),
    ];
    let conflict = PortConflict::capture(&world, 5173, &activity);
    world.activities.push(activity);

    let listeners = ListenerModel::new(
        &world,
        vec![ListenerRecord::new(
            5120,
            1,
            5173,
            "node (stale vite)",
            "sleeping",
            0,
            310,
        )?],
    )?;

    Ok(PortConflictCase {
        world,
        listeners: Some(listeners),
        conflict,
    })
}

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

#[expect(
    clippy::too_many_lines,
    reason = "One pinned fixture keeps related task and repository literals together for source comparison"
)]
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

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The fixed 410 MiB fixture is divided among exactly twelve resources; quotient plus at most one remains below its total"
)]
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
        hour.saturating_neg(),
    );
    seed.lines = vec![
        "psql: connection to server at \"db.internal\" failed".into(),
        "exit code 1".into(),
    ];
    w.activities = vec![dev, watch, deploy, seed];
}

fn parity_activity_stop_world(world: World) -> World {
    match seed_parity_activity_stop(world) {
        Ok(world) => world,
        Err(error) => parity_activity_stop_setup_failure(error),
    }
}

fn parity_activity_stop_setup_failure(error: ActivitySupervisorError) -> World {
    let scenario = Scenario::ParityActivityStop;
    let mut world = World::new(scenario, host_for(scenario), cwd_for(scenario));
    world.discovery = World::default_schedule();
    let mut diagnostic = Activity::new(
        0,
        "Activity stop fixture unavailable",
        cwd_for(scenario),
        ActivityState::Failed,
        0,
    );
    diagnostic.lines = vec![format!("Activity stop fixture unavailable · {error}")];
    world.activities.push(diagnostic);
    world
}

fn seed_parity_activity_stop(mut world: World) -> Result<World, ActivitySupervisorError> {
    let mut ordinary = Activity::new(
        1,
        "ordinary worker",
        "~/work/activity-stop/workers/ordinary",
        ActivityState::Running,
        0,
    );
    ordinary.lines = vec!["worker started".into(), "health check: ready".into()];
    world.register_scenario_activity(
        ordinary,
        ActivityExecutionStart::Running,
        ActivityStopProfile::Responsive,
    )?;

    let mut queued = Activity::new(
        2,
        "queued worker",
        "~/work/activity-stop/queue",
        ActivityState::Running,
        0,
    );
    queued.lines = vec!["waiting for a worker slot".into()];
    world.register_scenario_activity(
        queued,
        ActivityExecutionStart::Queued,
        ActivityStopProfile::Responsive,
    )?;

    let mut detached = Activity::new(
        3,
        "detached follower",
        "~/work/activity-stop/containers/api",
        ActivityState::Detached,
        0,
    );
    detached.lines = vec![
        "container log stream attached".into(),
        "service: api · state: healthy".into(),
    ];
    world.register_scenario_activity(
        detached,
        ActivityExecutionStart::Detached,
        ActivityStopProfile::Responsive,
    )?;

    let mut resistant = Activity::new(
        4,
        "resistant worker",
        "~/work/activity-stop/workers/resistant",
        ActivityState::Running,
        0,
    );
    resistant.lines = vec![
        "worker started".into(),
        "shutdown hook: waiting for response".into(),
    ];
    world.register_scenario_activity(
        resistant,
        ActivityExecutionStart::Running,
        ActivityStopProfile::Resistant,
    )?;

    Ok(world)
}

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

pub fn memory_command(
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
