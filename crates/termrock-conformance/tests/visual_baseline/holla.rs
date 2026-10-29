use crate::pointer::{wheel_below, wheel_down};
use crate::support::{self, Case, Color, HOLLA};

crate::baseline_case_with_variants!(
    holla_concept_first_use_default_120x40_truecolor => Case::new("holla/concept/first-use/120x40/truecolor", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/concept/first-use/80x24/truecolor", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/first-use/100x30/truecolor", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/first-use/120x40/truecolor", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/first-use/160x50/truecolor", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/first-use/100x30/none", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
        Case::new("holla/concept/first-use/72x20/truecolor", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 72, 20, Color::Truecolor, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_concept_monorepo_root_default_120x40_truecolor => Case::new("holla/concept/monorepo-root/120x40/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/concept/monorepo-root/80x24/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/monorepo-root/100x30/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/monorepo-root/120x40/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/monorepo-root/160x50/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/monorepo-root/100x30/none", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_concept_monorepo_child_default_120x40_truecolor => Case::new("holla/concept/monorepo-child/120x40/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/concept/monorepo-child/80x24/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/monorepo-child/100x30/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/monorepo-child/120x40/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/monorepo-child/160x50/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/monorepo-child/100x30/none", HOLLA, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_concept_docker_cleanup_default_120x40_truecolor => Case::new("holla/concept/docker-cleanup/120x40/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/concept/docker-cleanup/80x24/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/docker-cleanup/100x30/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/docker-cleanup/120x40/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/docker-cleanup/160x50/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/docker-cleanup/100x30/none", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);

crate::baseline_case_with_variants!(
    holla_concept_disk_cleanup_default_120x40_truecolor => Case::new("holla/concept/disk-cleanup/120x40/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/concept/disk-cleanup/80x24/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/disk-cleanup/100x30/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/disk-cleanup/120x40/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/disk-cleanup/160x50/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/disk-cleanup/100x30/none", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
        Case::new("holla/concept/disk-cleanup/100x30/256", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], 100, 30, Color::Ansi256, "holla❯"),
        Case::new("holla/concept/disk-cleanup/100x30/16", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], 100, 30, Color::Ansi16, "holla❯").timeout(20_000),
    ],
);

crate::baseline_case_with_variants!(
    holla_concept_activities_multi_default_120x40_truecolor => Case::new("holla/concept/activities-multi/120x40/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").timeout(20_000),
    [
        Case::new("holla/concept/activities-multi/80x24/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯").timeout(20_000),
        Case::new("holla/concept/activities-multi/100x30/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯").timeout(20_000),
        Case::new("holla/concept/activities-multi/120x40/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").timeout(20_000),
        Case::new("holla/concept/activities-multi/160x50/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯").timeout(20_000),
        Case::new("holla/concept/activities-multi/100x30/none", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯").timeout(20_000),
    ],
);
crate::baseline_case_with_variants!(
    holla_concept_remote_host_default_120x40_truecolor => Case::new("holla/concept/remote-host/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/concept/remote-host/80x24/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/remote-host/100x30/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/remote-host/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/remote-host/160x50/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/remote-host/100x30/none", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_concept_launch_failure_default_120x40_truecolor => Case::new("holla/concept/launch-failure/120x40/truecolor", HOLLA, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/concept/launch-failure/80x24/truecolor", HOLLA, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/launch-failure/100x30/truecolor", HOLLA, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/launch-failure/120x40/truecolor", HOLLA, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/launch-failure/160x50/truecolor", HOLLA, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/launch-failure/100x30/none", HOLLA, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_concept_hard_cases_default_120x40_truecolor => Case::new("holla/concept/hard-cases/120x40/truecolor", HOLLA, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/concept/hard-cases/80x24/truecolor", HOLLA, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/hard-cases/100x30/truecolor", HOLLA, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/hard-cases/120x40/truecolor", HOLLA, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/hard-cases/160x50/truecolor", HOLLA, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/concept/hard-cases/100x30/none", HOLLA, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
        Case::new("holla/concept/hard-cases/100x30/256", HOLLA, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 100, 30, Color::Ansi256, "holla❯"),
        Case::new("holla/concept/hard-cases/72x20/truecolor", HOLLA, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 72, 20, Color::Truecolor, "holla❯"),
    ],
);

crate::baseline_case_with_variants!(
    holla_parity_discovery_default_120x40_truecolor => Case::new("holla/parity/discovery/120x40/truecolor", HOLLA, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/discovery/80x24/truecolor", HOLLA, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/discovery/100x30/truecolor", HOLLA, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/discovery/120x40/truecolor", HOLLA, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/discovery/160x50/truecolor", HOLLA, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/discovery/100x30/none", HOLLA, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_history_default_120x40_truecolor => Case::new("holla/parity/history/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/history/80x24/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/history/100x30/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/history/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/history/160x50/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/history/100x30/none", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_files_default_120x40_truecolor => Case::new("holla/parity/files/120x40/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/files/80x24/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/files/100x30/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/files/120x40/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/files/160x50/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/files/100x30/none", HOLLA, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_browser_default_120x40_truecolor => Case::new("holla/parity/browser/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/browser/80x24/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/browser/100x30/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/browser/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/browser/160x50/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/browser/100x30/none", HOLLA, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_git_current_default_120x40_truecolor => Case::new("holla/parity/git-current/120x40/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/git-current/80x24/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/git-current/100x30/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/git-current/120x40/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/git-current/160x50/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/git-current/100x30/none", HOLLA, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_git_batch_default_120x40_truecolor => Case::new("holla/parity/git-batch/120x40/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/git-batch/80x24/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/git-batch/100x30/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/git-batch/120x40/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/git-batch/160x50/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/git-batch/100x30/none", HOLLA, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_task_sources_default_120x40_truecolor => Case::new("holla/parity/task-sources/120x40/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/task-sources/80x24/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/task-sources/100x30/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/task-sources/120x40/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/task-sources/160x50/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/task-sources/100x30/none", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_cargo_default_120x40_truecolor => Case::new("holla/parity/cargo/120x40/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/cargo/80x24/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/cargo/100x30/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/cargo/120x40/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/cargo/160x50/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/cargo/100x30/none", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_docker_default_120x40_truecolor => Case::new("holla/parity/docker/120x40/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/docker/80x24/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/docker/100x30/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/docker/120x40/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/docker/160x50/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/docker/100x30/none", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_brew_services_default_120x40_truecolor => Case::new("holla/parity/brew-services/120x40/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/brew-services/80x24/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/brew-services/100x30/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/brew-services/120x40/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/brew-services/160x50/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/brew-services/100x30/none", HOLLA, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_gradle_default_120x40_truecolor => Case::new("holla/parity/gradle/120x40/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/gradle/80x24/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/gradle/100x30/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/gradle/120x40/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/gradle/160x50/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/gradle/100x30/none", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_idea_default_120x40_truecolor => Case::new("holla/parity/idea/120x40/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/idea/80x24/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/idea/100x30/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/idea/120x40/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/idea/160x50/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/idea/100x30/none", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_upgrade_managers_default_120x40_truecolor => Case::new("holla/parity/upgrade-managers/120x40/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/upgrade-managers/80x24/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/upgrade-managers/100x30/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/upgrade-managers/120x40/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/upgrade-managers/160x50/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/upgrade-managers/100x30/none", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_executor_default_120x40_truecolor => Case::new("holla/parity/executor/120x40/truecolor", HOLLA, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/executor/80x24/truecolor", HOLLA, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/executor/100x30/truecolor", HOLLA, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/executor/120x40/truecolor", HOLLA, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/executor/160x50/truecolor", HOLLA, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/executor/100x30/none", HOLLA, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_task_input_default_120x40_truecolor => Case::new("holla/parity/task-input/120x40/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/task-input/80x24/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/task-input/100x30/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/task-input/120x40/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/task-input/160x50/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/task-input/100x30/none", HOLLA, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_custom_actions_default_120x40_truecolor => Case::new("holla/parity/custom-actions/120x40/truecolor", HOLLA, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/custom-actions/80x24/truecolor", HOLLA, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/custom-actions/100x30/truecolor", HOLLA, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/custom-actions/120x40/truecolor", HOLLA, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/custom-actions/160x50/truecolor", HOLLA, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/custom-actions/100x30/none", HOLLA, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_disk_scan_default_120x40_truecolor => Case::new("holla/parity/disk-scan/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/disk-scan/80x24/truecolor", HOLLA, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/disk-scan/100x30/truecolor", HOLLA, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/disk-scan/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/disk-scan/160x50/truecolor", HOLLA, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/disk-scan/100x30/none", HOLLA, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_disk_navigation_default_120x40_truecolor => Case::new("holla/parity/disk-navigation/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/disk-navigation/80x24/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/disk-navigation/100x30/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/disk-navigation/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/disk-navigation/160x50/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/disk-navigation/100x30/none", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_insights_default_120x40_truecolor => Case::new("holla/parity/insights/120x40/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/insights/80x24/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/insights/100x30/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/insights/120x40/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/insights/160x50/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/insights/100x30/none", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_delete_safety_default_120x40_truecolor => Case::new("holla/parity/delete-safety/120x40/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/delete-safety/80x24/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/delete-safety/100x30/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/delete-safety/120x40/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/delete-safety/160x50/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/delete-safety/100x30/none", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_cleanup_results_default_120x40_truecolor => Case::new("holla/parity/cleanup-results/120x40/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/cleanup-results/80x24/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/cleanup-results/100x30/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/cleanup-results/120x40/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/cleanup-results/160x50/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/cleanup-results/100x30/none", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_platforms_default_120x40_truecolor => Case::new("holla/parity/platforms/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/platforms/80x24/truecolor", HOLLA, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/platforms/100x30/truecolor", HOLLA, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/platforms/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/platforms/160x50/truecolor", HOLLA, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/platforms/100x30/none", HOLLA, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);
crate::baseline_case_with_variants!(
    holla_parity_platforms_linux_default_120x40_truecolor => Case::new("holla/parity/platforms-linux/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
    [
        Case::new("holla/parity/platforms-linux/80x24/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/platforms-linux/100x30/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 100, 30, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/platforms-linux/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/platforms-linux/160x50/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 160, 50, Color::Truecolor, "holla❯"),
        Case::new("holla/parity/platforms-linux/100x30/none", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 100, 30, Color::None, "holla❯"),
    ],
);

crate::baseline_case!(holla_flows_finder_query_120x40_truecolor => Case::new("holla/flows/finder/query/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:pull"]));
crate::baseline_case!(holla_flows_finder_query_selected_120x40_truecolor => Case::new("holla/flows/finder/query-selected/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:pull", "ctrl-a"]));
crate::baseline_case!(holla_flows_trust_prompt_120x40_truecolor => Case::new("holla/flows/trust/prompt/120x40/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:test", "enter"]));
crate::baseline_case!(holla_flows_files_results_120x40_truecolor => Case::new("holla/flows/files/results/120x40/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Find files under home", "enter", "type:readme"]));
crate::baseline_case!(holla_flows_files_unicode_120x40_truecolor => Case::new("holla/flows/files/unicode/120x40/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Find files under home", "enter", "ctrl-u", "type:café"]));
crate::baseline_case!(holla_flows_browser_hidden_120x40_truecolor => Case::new("holla/flows/browser/hidden/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "ctrl-h"]));
crate::baseline_case!(holla_flows_browser_preview_120x40_truecolor => Case::new("holla/flows/browser/preview/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "down", "down", "down"]));

crate::baseline_case!(holla_flows_files_preview_control_120x40_truecolor => Case::new("holla/flows/files/preview_control/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "down", "down", "down", "down", "down", "down", "down", "sleep:300"]));
crate::baseline_case!(holla_flows_cleanup_plan_120x40_truecolor => Case::new("holla/flows/cleanup/plan/120x40/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], 120, 40, Color::Truecolor, "holla❯").sends(&["c"]));
crate::baseline_case!(holla_flows_cleanup_gate_1_120x40_truecolor => Case::new("holla/flows/cleanup/gate-1/120x40/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], 120, 40, Color::Truecolor, "holla❯").sends(&["c", "c"]));
crate::baseline_case!(holla_flows_upgrade_excluded_120x40_truecolor => Case::new("holla/flows/upgrade/excluded/120x40/truecolor", HOLLA, &["--scenario", "upgrade-plan", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["down", "down", "down", "down", "down", "space"]));
crate::baseline_case!(holla_flows_upgrade_confirm_120x40_truecolor => Case::new("holla/flows/upgrade/confirm/120x40/truecolor", HOLLA, &["--scenario", "upgrade-plan", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["down", "down", "down", "down", "down", "space", "c"]));
crate::baseline_case!(holla_flows_remote_gate_1_120x40_truecolor => Case::new("holla/flows/remote/gate-1/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments", "enter"]));
crate::baseline_case!(holla_flows_help_overlay_120x40_truecolor => Case::new("holla/flows/help/overlay/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["f1"]));
crate::baseline_case!(holla_flows_activities_overlay_120x40_truecolor => Case::new("holla/flows/activities/overlay/120x40/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-g"]));

crate::baseline_case!(holla_flows_alternatives_120x40_truecolor => Case::new("holla/flows/alternatives/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:\u{1b}\r", "wait:Pin here"]));

crate::baseline_case!(holla_flows_activity_tab_jump_120x40_truecolor => Case::new("holla/flows/activities/jump/120x40/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["alt-1"]));

crate::baseline_case!(holla_flows_menu_file_120x40_truecolor => Case::new("holla/flows/menu/file/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["f10", "wait:Alternatives…"]));
crate::baseline_case!(holla_flows_menu_go_120x40_truecolor => Case::new("holla/flows/menu/go/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["f10", "right", "wait:Parent scope"]));

crate::baseline_case!(holla_flows_remote_gate2_120x40_truecolor => Case::new("holla/flows/remote/gate2/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type RESTART PAYMENTS ON prod-eu-1"]));

crate::baseline_case!(holla_flows_remote_gate2_typed_120x40_truecolor => Case::new("holla/flows/remote/gate2_typed/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type RESTART PAYMENTS ON prod-eu-1", "enter", "type:RESTART PAYMENTS ON prod-eu-1"]));

crate::baseline_case!(holla_flows_scope_parent_120x40_truecolor => Case::new("holla/flows/scope/parent/120x40/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-up", "wait:No project root above"]));
crate::baseline_case!(holla_flows_scope_children_120x40_truecolor => Case::new("holla/flows/scope/children/120x40/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-down", "wait:Children · 3 projects"]));
crate::baseline_case!(holla_flows_scope_system_120x40_truecolor => Case::new("holla/flows/scope/system/120x40/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-up", "ctrl-up", "wait:scope ‹ system ›"]));

crate::baseline_case!(holla_flows_query_undone_120x40_truecolor => Case::new("holla/flows/query/undone/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:pull", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z"]));
crate::baseline_case!(holla_flows_query_redone_120x40_truecolor => Case::new("holla/flows/query/redone/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:pull", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-y", "ctrl-y", "ctrl-y", "ctrl-y"]));

crate::baseline_case!(holla_flows_files_jump_120x40_truecolor => Case::new("holla/flows/files/jump/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path"]));
crate::baseline_case!(holla_flows_files_jump_error_120x40_truecolor => Case::new("holla/flows/files/jump_error/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path", "type:nope", "enter", "wait:no such file or directory"]));

crate::baseline_case!(holla_flows_activities_empty_120x40_truecolor => Case::new("holla/flows/activities/empty/120x40/truecolor", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-g", "wait:Nothing has been started yet"]));

crate::baseline_case!(holla_flows_quit_confirm_120x40_truecolor => Case::new("holla/flows/quit/confirm/120x40/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-q", "wait:still running"]));

crate::baseline_case!(holla_flows_trust_accepted_120x40_truecolor => Case::new("holla/flows/trust/accepted/120x40/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:test", "enter", "wait:Trust ~/work/acme/apps/frontend/mise.toml?", "right", "enter", "wait:Trusted ~/work/acme/apps/frontend/mise.toml"]));

crate::baseline_case!(holla_flows_args_page_120x40_truecolor => Case::new("holla/flows/args/page/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port"]));

crate::baseline_case!(holla_flows_args_clone_120x40_truecolor => Case::new("holla/flows/args/clone/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:clone", "enter", "wait:Clone a GitHub repository"]));

crate::baseline_case!(holla_flows_docker_gate2_typed_120x40_truecolor => Case::new("holla/flows/docker/gate2_typed/120x40/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox"]));

crate::baseline_case!(holla_flows_docker_done_120x40_truecolor => Case::new("holla/flows/docker/done/120x40/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:Plan changed", "right", "enter", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:7 succeeded"]).timeout(60_000));

crate::baseline_case!(holla_flows_disk_top_files_120x40_truecolor => Case::new("holla/flows/disk/top_files/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "t", "wait:Top files · Spotlight"]).timeout(15_000));

crate::baseline_case!(holla_flows_cleanup_report_120x40_truecolor => Case::new("holla/flows/cleanup/report/120x40/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:TRASH 1 UNDER /Users/alex/Projects/safe ON mbp", "enter", "tab", "enter", "wait:Cleanup report"]).timeout(15_000));

crate::baseline_case!(holla_flows_child_query_120x40_truecolor => Case::new("holla/flows/child/query/120x40/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:test", "wait:Results · 17"]));

crate::baseline_case!(holla_flows_query_empty_preview_120x40_truecolor => Case::new("holla/flows/query/empty_preview/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "escape", "escape", "type:blocking", "enter", "wait:Who is blocking", "escape", "type:resources", "wait:No matches for"]));

crate::baseline_case!(holla_flows_remote_query_120x40_truecolor => Case::new("holla/flows/remote/query/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments"]));

crate::baseline_case!(holla_flows_port_snapshot_120x40_truecolor => Case::new("holla/flows/args/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port", "ctrl-s", "wait:Port 5173"]));

crate::baseline_case!(holla_flows_upgrade_facts_80x24_truecolor => Case::new("holla/flows/upgrade/facts/80x24/truecolor", HOLLA, &["--scenario", "upgrade-plan", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯").sends(&["p", "wait:Preflight"]));

crate::baseline_case!(holla_flows_config_page_120x40_truecolor => Case::new("holla/flows/config/page/120x40/truecolor", HOLLA, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Custom action configuration", "enter", "wait:sha256:"]));

crate::baseline_case!(holla_flows_files_actions_120x40_truecolor => Case::new("holla/flows/files/actions/120x40/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Find files under home", "enter", "wait:of 29 indexed", "ctrl-u", "type:todo", "wait:1 result", "enter", "wait:Open in the OS"]));

crate::baseline_case!(holla_flows_files_jumped_120x40_truecolor => Case::new("holla/flows/files/jumped/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path", "type:~/work", "enter", "wait:Jumped to ~/work"]));

crate::baseline_case!(holla_flows_git_pull_blocked_120x40_truecolor => Case::new("holla/flows/git/pull_blocked/120x40/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull", "wait:blocked · 1 modified"]));
crate::baseline_case!(holla_flows_git_merge_120x40_truecolor => Case::new("holla/flows/git/merge/120x40/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull with merge", "enter", "right", "enter", "wait:error: Your local changes"]));
crate::baseline_case!(holla_flows_git_push_rejected_120x40_truecolor => Case::new("holla/flows/git/push_rejected/120x40/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull with merge", "enter", "right", "enter", "wait:error: Your local changes", "alt-0", "ctrl-u", "type:Push", "enter", "right", "enter", "wait:rejected"]));

crate::baseline_case!(holla_flows_git_batch_120x40_truecolor => Case::new("holla/flows/git/batch/120x40/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull 4 sibling", "enter", "right", "enter", "wait:Fast-forward"]).timeout(15_000));
crate::baseline_case!(holla_flows_git_batch_picker_120x40_truecolor => Case::new("holla/flows/git/batch_picker/120x40/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull 4 sibling", "enter", "right", "enter", "wait:Fast-forward", "wait:2 ok · 2 failed", "ctrl-g", "wait:0 live"]).timeout(15_000));

crate::baseline_case!(holla_flows_task_sources_120x40_truecolor => Case::new("holla/flows/task/sources/120x40/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:yarn", "wait:yarn s00"]));
crate::baseline_case!(holla_flows_task_sources_diagnostic_120x40_truecolor => Case::new("holla/flows/task/sources_diagnostic/120x40/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Taskfile", "wait:discovery diagnostic"]));

crate::baseline_case!(holla_flows_cargo_clean_120x40_truecolor => Case::new("holla/flows/cargo/clean/120x40/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:cargo clean", "wait:900.0 MiB"]));
crate::baseline_case!(holla_flows_cargo_clean_confirm_120x40_truecolor => Case::new("holla/flows/cargo/clean_confirm/120x40/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:cargo clean", "enter", "wait:Esc Cancel"]));

crate::baseline_case!(holla_flows_docker_remove_failed_120x40_truecolor => Case::new("holla/flows/docker/remove_failed/120x40/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Stop all containers", "enter", "right", "enter", "wait:exit 1"]));
crate::baseline_case!(holla_flows_docker_remove_gate1_120x40_truecolor => Case::new("holla/flows/docker/remove_gate1/120x40/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Stop and remove all", "enter", "wait:gate 1 of 2"]));
crate::baseline_case!(holla_flows_docker_remove_gate2_120x40_truecolor => Case::new("holla/flows/docker/remove_gate2/120x40/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Stop and remove all", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type REMOVE ALL CONTAINERS ON mbp"]));

crate::baseline_case!(holla_flows_brew_services_120x40_truecolor => Case::new("holla/flows/brew/services/120x40/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Homebrew services", "enter", "wait:svc02"]));
crate::baseline_case!(holla_flows_brew_services_stop_failed_120x40_truecolor => Case::new("holla/flows/brew/services_stop_failed/120x40/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Stop svc02", "enter", "right", "enter", "wait:Bootstrap failed"]));

crate::baseline_case!(holla_flows_gradle_cleanup_120x40_truecolor => Case::new("holla/flows/gradle/cleanup/120x40/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Clean Gradle outputs", "enter", "wait:Cleanup · Gradle outputs", "right", "sleep:300"]));
crate::baseline_case!(holla_flows_gradle_gate1_120x40_truecolor => Case::new("holla/flows/gradle/gate1/120x40/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Clean Gradle outputs", "enter", "wait:Cleanup · Gradle outputs", "right", "d", "sleep:300"]));
crate::baseline_case!(holla_flows_idea_cleanup_120x40_truecolor => Case::new("holla/flows/idea/cleanup/120x40/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Clean IntelliJ", "enter", "right", "wait:idea.clean"]));
crate::baseline_case!(holla_flows_idea_gate2_120x40_truecolor => Case::new("holla/flows/idea/gate2/120x40/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Clean IntelliJ", "enter", "right", "wait:idea.clean", "d", "wait:gate 1 of 2", "right", "enter", "wait:Type TRASH 4 UNDER /Users/alex/work/ide ON mbp"]));

crate::baseline_case!(holla_flows_brew_upgrade_batch_120x40_truecolor => Case::new("holla/flows/brew/upgrade_batch/120x40/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Upgrade Homebrew packages", "enter", "right", "enter", "wait:Outdated Formulae · 2"]).timeout(15_000));
crate::baseline_case!(holla_flows_brew_upgrade_plan_120x40_truecolor => Case::new("holla/flows/brew/upgrade_plan/120x40/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Upgrade Homebrew packages", "enter", "right", "enter", "wait:Outdated Formulae · 2", "alt-0", "ctrl-u", "type:Upgrade everything", "enter", "wait:9 steps"]).timeout(15_000));

crate::baseline_case!(holla_flows_task_input_cancelled_120x40_truecolor => Case::new("holla/flows/task/input_cancelled/120x40/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Deploy the release", "enter", "right", "enter", "wait:Password:", "i", "type:hunter2", "enter", "wait:Proceed with rollout?", "type:n", "enter", "wait:rollout cancelled by operator"]).timeout(15_000));
crate::baseline_case!(holla_flows_task_input_killed_120x40_truecolor => Case::new("holla/flows/task/input_killed/120x40/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Deploy the release", "enter", "right", "enter", "wait:Password:", "i", "type:hunter2", "enter", "wait:Proceed with rollout?", "type:n", "enter", "wait:rollout cancelled by operator", "alt-0", "ctrl-u", "type:Run the stubborn worker", "enter", "wait:ignoring SIGTERM", "s", "wait:killed (SIGKILL)"]).timeout(15_000));

crate::baseline_case!(holla_flows_trusted_deploy_failed_120x40_truecolor => Case::new("holla/flows/trust/deploy_failed/120x40/truecolor", HOLLA, &["--scenario", "parity-custom-actions", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Deploy preview", "enter", "wait:Trust ~/work/team/.holla.toml?", "right", "enter", "wait:exit 127"]));

crate::baseline_case!(holla_flows_disk_scan_complete_120x40_truecolor => Case::new("holla/flows/disk/scan_complete/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-scan", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:2 unreadable"]).timeout(15_000));

crate::baseline_case!(holla_flows_disk_tree_120x40_truecolor => Case::new("holla/flows/disk/tree/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete"]).timeout(15_000));
crate::baseline_case!(holla_flows_disk_tree_apparent_120x40_truecolor => Case::new("holla/flows/disk/tree_apparent/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "wait:largest first · apparent"]).timeout(15_000));
crate::baseline_case!(holla_flows_disk_tree_unfolded_120x40_truecolor => Case::new("holla/flows/disk/tree_unfolded/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "s", "down", "down", "f", "wait:allocated · unfolded"]).timeout(15_000));
crate::baseline_case!(holla_flows_disk_tree_selected_120x40_truecolor => Case::new("holla/flows/disk/tree_selected/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "s", "down", "down", "f", "f", "space", "wait:1 selected (176.0"]).timeout(15_000));

crate::baseline_case!(holla_flows_cleanup_categories_120x40_truecolor => Case::new("holla/flows/cleanup/categories/120x40/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories"]));
crate::baseline_case!(holla_flows_cleanup_derived_data_120x40_truecolor => Case::new("holla/flows/cleanup/derived_data/120x40/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories", "right", "wait:▾ Xcode DerivedData"]));
crate::baseline_case!(holla_flows_cleanup_artifacts_120x40_truecolor => Case::new("holla/flows/cleanup/artifacts/120x40/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories", "end", "wait:Project artifacts"]));

crate::baseline_case!(holla_flows_cleanup_gate1_120x40_truecolor => Case::new("holla/flows/cleanup/gate1/120x40/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2"]).timeout(15_000));
crate::baseline_case!(holla_flows_cleanup_gate2_120x40_truecolor => Case::new("holla/flows/cleanup/gate2/120x40/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2"]).timeout(15_000));
crate::baseline_case!(holla_flows_cleanup_gate2_typed_120x40_truecolor => Case::new("holla/flows/cleanup/gate2_typed/120x40/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:TRASH 1 UNDER /Users/alex/Projects/safe ON mbp"]).timeout(15_000));

crate::baseline_case!(holla_flows_cleanup_history_120x40_truecolor => Case::new("holla/flows/cleanup/history/120x40/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Show cleanup history", "enter", "wait:operation log records"]));

crate::baseline_case!(holla_flows_platforms_linux_gate1_120x40_truecolor => Case::new("holla/flows/platforms/linux_gate1/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend"]));
crate::baseline_case!(holla_flows_platforms_linux_gate2_120x40_truecolor => Case::new("holla/flows/platforms/linux_gate2/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend", "right", "enter", "wait:Type TRASH 1 UNDER /home/alex ON devbox"]));

crate::baseline_case!(holla_flows_cleanup_linux_report_120x40_truecolor => Case::new("holla/flows/cleanup/linux_report/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend", "right", "enter", "wait:Type TRASH 1 UNDER /home/alex ON devbox", "enter", "type:TRASH 1 UNDER /home/alex ON devbox", "enter", "tab", "enter", "wait:Cleanup report"]).timeout(15_000));
crate::baseline_case!(holla_flows_mac_top_files_120x40_truecolor => Case::new("holla/flows/platforms/files/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Top files on this Mac", "enter", "wait:did not finish within 5 s"]));

crate::baseline_case!(holla_flows_docker_drift_120x40_truecolor => Case::new("holla/flows/docker/drift/120x40/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:a new container"]));

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_fade_trust_body_wheel_matrix() {
    let case = Case::new(
        "holla/fade/trust_body_wheel/120x40/truecolor",
        HOLLA,
        &["--scenario", "parity-custom-actions", "--motion", "reduced"],
        120,
        40,
        Color::Truecolor,
        "holla❯",
    )
    .sends(&[
        "type:Deploy preview",
        "enter",
        "wait:Trust ~/work/team/.holla.toml?",
    ]);
    support::run_canonical_live(&case, |s, variant| {
        if variant.rows <= 24 {
            wheel_down(s, "Trust scope", 2);
        } else {
            wheel_down(s, "id = \"deploy.preview\"", 2);
        }
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_fade_cleanup_list_wheel_matrix() {
    let case = Case::new(
        "holla/fade/cleanup_list_wheel/120x40/truecolor",
        HOLLA,
        &["--scenario", "parity-insights", "--motion", "reduced"],
        120,
        40,
        Color::Truecolor,
        "holla❯",
    )
    .sends(&[
        "type:Review cleanup candidates",
        "enter",
        "wait:18 categories",
    ]);
    support::run_canonical_live(&case, |s, case| {
        if case.rows <= 24 {
            wheel_below(s, "Xcode DerivedData", 2);
        } else {
            wheel_below(s, "Yarn cache", 2);
        }
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_fade_executor_burst_end_matrix() {
    let case = Case::new(
        "holla/fade/executor_burst-end/120x40/truecolor",
        HOLLA,
        &["--scenario", "parity-executor", "--motion", "reduced"],
        120,
        40,
        Color::Truecolor,
        "holla❯",
    )
    .sends(&[
        "type:Emit a burst",
        "enter",
        "wait:500 earlier lines dropped",
    ]);
    support::run_canonical_live(&case, |_, _| ());
}
