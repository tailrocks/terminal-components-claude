//! holla captures (253): 9 concept scenarios under `holla/concept/`, 23 parity
//! scenarios under `holla/parity/`, journeys under `holla/flows/` (including
//! the H1-H21 chord/gate/journey additions), wheel scroll-fade under
//! `holla/fade/`. The two audit fixtures (rust-dirty, upgrade-plan) keep
//! statics only in audit.rs (`holla/audit/`, 5x5 matrix - the dedupe rule).
//!
//! Ported verbatim from the retired tools/tuisnap_baseline.sh (argv, needles,
//! sends, CAP_TIMEOUTs); only the store names were regrouped. Do not
//! hand-tune: drift against the approved frames means the port or the app
//! changed.
//!
//! Motion for the new flows follows the determinism playbook: paused+frame
//! whenever the captured frame can carry a tick-derived label (`live · N s
//! ago`, the snapshot pages' `· live ·` meta, activity spinners); reduced only
//! where ticks must advance the world (files indexing, disk scan, the docker
//! plan run, the executor burst).

use std::time::Duration;

use tuisnap::pty::{Scroll, Session};

use crate::pointer::wheel_below;
use crate::support::{self, Case, Color, HOLLA};

// ---------------------------------------------------------------- concept --

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
// The 16-colour probe proved this boot stream can outlive the default at
// exactly 100x30/16; retain that timeout only for that exact combo.
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
// activities-multi's fast-forward (5 live activities × 40 ticks) is the
// slowest boot in the suite; under the full-suite 8-thread load the default
// 8 s boot budget can expire before the first frame (empty-screen timeouts).
// A 20 s boot budget — the pixel gate is untouched.
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

// concept extras (verbatim remap of the ad-hoc legacy combos)

// ----------------------------------------------------------------- parity --

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

// ------------------------------------------------------------------ flows --

// paused, not reduced like the other journeys: parity-history renders a
// wall-clock relative-time label (`live · N s ago`) that reduced-motion ticks
// advance — the pinned frame keeps it deterministic (same contract as
// finder_query-selected below).
crate::baseline_case!(holla_flows_finder_query_120x40_truecolor => Case::new("holla/flows/finder/query/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:pull"]));
crate::baseline_case!(holla_flows_finder_query_selected_120x40_truecolor => Case::new("holla/flows/finder/query-selected/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:pull", "ctrl-a"]));
crate::baseline_case!(holla_flows_trust_prompt_120x40_truecolor => Case::new("holla/flows/trust/prompt/120x40/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:test", "enter"]));
crate::baseline_case!(holla_flows_files_results_120x40_truecolor => Case::new("holla/flows/files/results/120x40/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Find files under home", "enter", "type:readme"]));
crate::baseline_case!(holla_flows_files_unicode_120x40_truecolor => Case::new("holla/flows/files/unicode/120x40/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Find files under home", "enter", "ctrl-u", "type:café"]));
crate::baseline_case!(holla_flows_browser_hidden_120x40_truecolor => Case::new("holla/flows/browser/hidden/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "ctrl-h"]));
crate::baseline_case!(holla_flows_browser_preview_120x40_truecolor => Case::new("holla/flows/browser/preview/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "down", "down", "down"]));
// h_hp04_preview_control: same listing as browser_preview (big.log at 3 downs);
// control.txt is 7 downs (dirs first). Preview sanitises ESC/BEL to �.
crate::baseline_case!(holla_flows_files_preview_control_120x40_truecolor => Case::new("holla/flows/files/preview_control/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "down", "down", "down", "down", "down", "down", "down", "sleep:300"]));
crate::baseline_case!(holla_flows_cleanup_plan_120x40_truecolor => Case::new("holla/flows/cleanup/plan/120x40/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], 120, 40, Color::Truecolor, "holla❯").sends(&["c"]));
crate::baseline_case!(holla_flows_cleanup_gate_1_120x40_truecolor => Case::new("holla/flows/cleanup/gate-1/120x40/truecolor", HOLLA, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], 120, 40, Color::Truecolor, "holla❯").sends(&["c", "c"]));
crate::baseline_case!(holla_flows_upgrade_excluded_120x40_truecolor => Case::new("holla/flows/upgrade/excluded/120x40/truecolor", HOLLA, &["--scenario", "upgrade-plan", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["down", "down", "down", "down", "down", "space"]));
crate::baseline_case!(holla_flows_upgrade_confirm_120x40_truecolor => Case::new("holla/flows/upgrade/confirm/120x40/truecolor", HOLLA, &["--scenario", "upgrade-plan", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["down", "down", "down", "down", "down", "space", "c"]));
crate::baseline_case!(holla_flows_remote_gate_1_120x40_truecolor => Case::new("holla/flows/remote/gate-1/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments", "enter"]));
crate::baseline_case!(holla_flows_help_overlay_120x40_truecolor => Case::new("holla/flows/help/overlay/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["f1"]));
crate::baseline_case!(holla_flows_activities_overlay_120x40_truecolor => Case::new("holla/flows/activities/overlay/120x40/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-g"]));

// ------------------------------------------------------- flows (new, H1-H21) --

// H1: Alt+Enter alternatives menu. `alt-enter` is not chord-expressible, so
// the bytes a real terminal sends for it are typed verbatim in one write
// (`\x1b\r`, the pointer.rs verbatim technique). Paused: the preview panel
// behind the menu carries a `live · N s ago` label.
crate::baseline_case!(holla_flows_alternatives_120x40_truecolor => Case::new("holla/flows/alternatives/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:\u{1b}\r", "wait:Pin here"]));
// H2: Alt+1 jumps to the second tab (frontend dev); the strip's spinner
// glyphs are tick frames, so the clock stays frozen.
crate::baseline_case!(holla_flows_activity_tab_jump_120x40_truecolor => Case::new("holla/flows/activities/jump/120x40/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["alt-1"]));
// H3/H4: F10 opens the File menu, Right crosses to Go (the
// app_tests_flows.rs menu proof, PTY form).
crate::baseline_case!(holla_flows_menu_file_120x40_truecolor => Case::new("holla/flows/menu/file/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["f10", "wait:Alternatives…"]));
crate::baseline_case!(holla_flows_menu_go_120x40_truecolor => Case::new("holla/flows/menu/go/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["f10", "right", "wait:Parent scope"]));
// H5/H6: the two-gate remote action. Gate 1 continues with Right+Enter (its
// `y` copies the sequence, review.rs) — the spec's `y` was a misread. Paused:
// the gate-1 facts render `live · N s ago`.
crate::baseline_case!(holla_flows_remote_gate2_120x40_truecolor => Case::new("holla/flows/remote/gate2/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type RESTART PAYMENTS ON prod-eu-1"]));
// H6: Enter arms the ack field for editing, then the host-bound phrase is
// typed; the dialog stays open with the armed token visible.
crate::baseline_case!(holla_flows_remote_gate2_typed_120x40_truecolor => Case::new("holla/flows/remote/gate2_typed/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type RESTART PAYMENTS ON prod-eu-1", "enter", "type:RESTART PAYMENTS ON prod-eu-1"]));
// H7-H9: scope chords from the monorepo root (the scope walk of
// app_tests_flows.rs). Key-driven, and paused pins the preview panel's
// `live · N s ago` freshness label (the finder_query lesson; the
// system-scope preview demonstrably carries it).
crate::baseline_case!(holla_flows_scope_parent_120x40_truecolor => Case::new("holla/flows/scope/parent/120x40/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-up", "wait:No project root above"]));
crate::baseline_case!(holla_flows_scope_children_120x40_truecolor => Case::new("holla/flows/scope/children/120x40/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-down", "wait:Children · 3 projects"]));
crate::baseline_case!(holla_flows_scope_system_120x40_truecolor => Case::new("holla/flows/scope/system/120x40/truecolor", HOLLA, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-up", "ctrl-up", "wait:scope ‹ system ›"]));
// H10/H11: query undo/redo on the same paused fixture as finder_query (the
// history preview's `live · N s ago` froze that pair too). Undo is
// per-keystroke: emptying `pull` takes four Ctrl+Z, restoring it four
// Ctrl+Y.
crate::baseline_case!(holla_flows_query_undone_120x40_truecolor => Case::new("holla/flows/query/undone/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:pull", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z"]));
crate::baseline_case!(holla_flows_query_redone_120x40_truecolor => Case::new("holla/flows/query/redone/120x40/truecolor", HOLLA, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:pull", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-y", "ctrl-y", "ctrl-y", "ctrl-y"]));
// H12/H13: the Go-to-path picker and its validation error. Reduced: the
// files page's pending listing only applies on a tick (paused would wedge it
// animating); the picker and the error carry no tick label.
crate::baseline_case!(holla_flows_files_jump_120x40_truecolor => Case::new("holla/flows/files/jump/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path"]));
crate::baseline_case!(holla_flows_files_jump_error_120x40_truecolor => Case::new("holla/flows/files/jump_error/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path", "type:nope", "enter", "wait:no such file or directory"]));
// H14: Ctrl+G with nothing started (the h_flow_activities_empty state, on
// the spec's first-use fixture).
crate::baseline_case!(holla_flows_activities_empty_120x40_truecolor => Case::new("holla/flows/activities/empty/120x40/truecolor", HOLLA, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-g", "wait:Nothing has been started yet"]));
// H15: Ctrl+Q with live activities opens the quit confirmation.
crate::baseline_case!(holla_flows_quit_confirm_120x40_truecolor => Case::new("holla/flows/quit/confirm/120x40/truecolor", HOLLA, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["ctrl-q", "wait:still running"]));
// H16: trust accepted (Cancel is the default focus; Right arms Trust). The
// accept runs the task, which lands on its arguments page — the post-accept
// state; the `Trusted …` status line outlives the settle (5 virtual seconds).
crate::baseline_case!(holla_flows_trust_accepted_120x40_truecolor => Case::new("holla/flows/trust/accepted/120x40/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:test", "enter", "wait:Trust ~/work/acme/apps/frontend/mise.toml?", "right", "enter", "wait:Trusted ~/work/acme/apps/frontend/mise.toml"]));
// H17: an item with structured args opens the ArgsPage instead of running
// (`port` ranks the port action first — the catalog's exact-keyword proof).
crate::baseline_case!(holla_flows_args_page_120x40_truecolor => Case::new("holla/flows/args/page/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port"]));
// h_flow_args: clone-repo Arguments page (`clone` ranks github.clone first).
// rust-dirty (not monorepo-child): destination ~/work/holla, owner alex-dev.
crate::baseline_case!(holla_flows_args_clone_120x40_truecolor => Case::new("holla/flows/args/clone/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:clone", "enter", "wait:Clone a GitHub repository"]));
// H18: docker gate 2 with the target-bound phrase typed (h_p3_docker's
// gate2_typed state). All key-driven; paused keeps the gate-1 `Provenance ·
// N s ago` label frozen.
crate::baseline_case!(holla_flows_docker_gate2_typed_120x40_truecolor => Case::new("holla/flows/docker/gate2_typed/120x40/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox"]));
// H19: the full run. The first Execute meets the plan drift and bounces back
// to gate 1 (the docker_cleanup_takes_two_gates flow); the second pass
// executes and ticks drive the plan to Done. Reduced is required (the run is
// tick-driven); the end frame is stable because nothing animates after Done.
crate::baseline_case!(holla_flows_docker_done_120x40_truecolor => Case::new("holla/flows/docker/done/120x40/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:Plan changed", "right", "enter", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:7 succeeded"]).timeout(60_000));
// H20: the Spotlight top-files page over the finished scan (`t` opens it,
// disk.rs). Reduced: the tree scan completes only through ticks.
crate::baseline_case!(holla_flows_disk_top_files_120x40_truecolor => Case::new("holla/flows/disk/top_files/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "t", "wait:Top files · Spotlight"]).timeout(15_000));
// H21: a cleanup report. The spec's `down…, enter` was a misread: the
// history page has no reports at boot (nothing links one), so Tab fell
// through to the menu bar. The real route is the hp22 flow: select `big`
// (the largest child, one row under the root) in the finished scan, `d`,
// both gates, and the commit pushes the report page — one tick later it is
// the finished Cleanup report. Reduced: the scan and the cleanup job are
// tick-driven; the page's `· live ·` meta is a constant ("1 s ago"), not a
// tick readout. `space` selects the row first — `d` refuses an empty
// selection ("Select at least one entry first").
crate::baseline_case!(holla_flows_cleanup_report_120x40_truecolor => Case::new("holla/flows/cleanup/report/120x40/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:TRASH 1 UNDER /Users/alex/Projects/safe ON mbp", "enter", "tab", "enter", "wait:Cleanup report"]).timeout(15_000));

// ------------------------------------------- coverage holes (legacy h_flow_*,
// h_hp*, h_p3 states; keys from the retired tools/holla_flows.sh and
// tools/holla_parity_flows.sh, motion hardened per the determinism playbook) --

// h_flow_child_query: the `test` query in the monorepo child (the
// trust_prompt journey's pre-Enter state).
crate::baseline_case!(holla_flows_child_query_120x40_truecolor => Case::new("holla/flows/child/query/120x40/truecolor", HOLLA, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:test", "wait:Results · 17"]));
// h_flow_system: a query with no matches leaves an empty preview; the
// `Cancelled` status comes from leaving gate 1 (the legacy chain typed
// `blocking`, opened the blocker page, then appended `resources`).
crate::baseline_case!(holla_flows_query_empty_preview_120x40_truecolor => Case::new("holla/flows/query/empty_preview/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "escape", "escape", "type:blocking", "enter", "wait:Who is blocking", "escape", "type:resources", "wait:No matches for"]));
// h_flow_remote_query: the remote query typed, not yet run.
crate::baseline_case!(holla_flows_remote_query_120x40_truecolor => Case::new("holla/flows/remote/query/120x40/truecolor", HOLLA, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:restart payments"]));
// h_flow_port_snapshot: the args flow's end state — Ctrl+S takes the
// defaults (port 5173) and opens the listeners snapshot.
crate::baseline_case!(holla_flows_port_snapshot_120x40_truecolor => Case::new("holla/flows/args/120x40/truecolor", HOLLA, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port", "ctrl-s", "wait:Port 5173"]));
// h_flow_upgrade_80_facts: the plan step-facts drawer at the minimum size
// (`p` toggles it, plan.rs).
crate::baseline_case!(holla_flows_upgrade_facts_80x24_truecolor => Case::new("holla/flows/upgrade/facts/80x24/truecolor", HOLLA, &["--scenario", "upgrade-plan", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "holla❯").sends(&["p", "wait:Preflight"]));

// h_hp01_config_page: the custom-action configuration diagnostics page.
crate::baseline_case!(holla_flows_config_page_120x40_truecolor => Case::new("holla/flows/config/page/120x40/truecolor", HOLLA, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Custom action configuration", "enter", "wait:sha256:"]));
// h_hp02_recent — DROP: the usage-populated Recent list ("used N times
// here") exists only with the history store; the suite sets
// HOLLA_NO_HISTORY=1 by design (support.rs opts_for — ambient-learning
// hygiene), so every capture shows "nothing used here yet". The populated
// state is unreachable by construction, not by nondeterminism.
// h_hp03_actions: Enter on a find result opens its actions popup.
crate::baseline_case!(holla_flows_files_actions_120x40_truecolor => Case::new("holla/flows/files/actions/120x40/truecolor", HOLLA, &["--scenario", "parity-files", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Find files under home", "enter", "wait:of 29 indexed", "ctrl-u", "type:todo", "wait:1 result", "enter", "wait:Open in the OS"]));
// h_hp04_jumped: the browser after a successful Go-to-path jump.
crate::baseline_case!(holla_flows_files_jumped_120x40_truecolor => Case::new("holla/flows/files/jumped/120x40/truecolor", HOLLA, &["--scenario", "parity-browser", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path", "type:~/work", "enter", "wait:Jumped to ~/work"]));
// h_hp05: git-current — the blocked pull preview, the failed merge and the
// rejected push (push_rejected keeps the merge's failed tab, like the
// legacy single-session chain).
crate::baseline_case!(holla_flows_git_pull_blocked_120x40_truecolor => Case::new("holla/flows/git/pull_blocked/120x40/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull", "wait:blocked · 1 modified"]));
crate::baseline_case!(holla_flows_git_merge_120x40_truecolor => Case::new("holla/flows/git/merge/120x40/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull with merge", "enter", "right", "enter", "wait:error: Your local changes"]));
crate::baseline_case!(holla_flows_git_push_rejected_120x40_truecolor => Case::new("holla/flows/git/push_rejected/120x40/truecolor", HOLLA, &["--scenario", "parity-git-current", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull with merge", "enter", "right", "enter", "wait:error: Your local changes", "alt-0", "ctrl-u", "type:Push", "enter", "right", "enter", "wait:rejected"]));
// h_hp06: the sibling batch run to completion, then the activities picker
// over it (2 succeeded, 2 failed).
crate::baseline_case!(holla_flows_git_batch_120x40_truecolor => Case::new("holla/flows/git/batch/120x40/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull 4 sibling", "enter", "right", "enter", "wait:Fast-forward"]).timeout(15_000));
crate::baseline_case!(holla_flows_git_batch_picker_120x40_truecolor => Case::new("holla/flows/git/batch_picker/120x40/truecolor", HOLLA, &["--scenario", "parity-git-batch", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Pull 4 sibling", "enter", "right", "enter", "wait:Fast-forward", "wait:2 ok · 2 failed", "ctrl-g", "wait:0 live"]).timeout(15_000));
// h_hp07: the capped script list and the discovery-diagnostic row.
crate::baseline_case!(holla_flows_task_sources_120x40_truecolor => Case::new("holla/flows/task/sources/120x40/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:yarn", "wait:yarn s00"]));
crate::baseline_case!(holla_flows_task_sources_diagnostic_120x40_truecolor => Case::new("holla/flows/task/sources_diagnostic/120x40/truecolor", HOLLA, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Taskfile", "wait:discovery diagnostic"]));
// h_hp08: the cargo clean row and its one-confirmation dialog.
crate::baseline_case!(holla_flows_cargo_clean_120x40_truecolor => Case::new("holla/flows/cargo/clean/120x40/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:cargo clean", "wait:900.0 MiB"]));
crate::baseline_case!(holla_flows_cargo_clean_confirm_120x40_truecolor => Case::new("holla/flows/cargo/clean_confirm/120x40/truecolor", HOLLA, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:cargo clean", "enter", "wait:Esc Cancel"]));
// h_hp09: the daemon-stage stop-all failure, then the remove-all gates.
// The gates are taken from a fresh boot under paused: the legacy chain
// reached them over the failed tab, but that prefix leaves the gate's
// `Data · live N s ago` re-resolve straddling a second boundary under
// full-suite load (0/1 s flip-flop, observed). Paused pins the label (and
// renders "2 s ago" like the legacy frame); the failure itself is covered
// by docker_remove_failed.
crate::baseline_case!(holla_flows_docker_remove_failed_120x40_truecolor => Case::new("holla/flows/docker/remove_failed/120x40/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Stop all containers", "enter", "right", "enter", "wait:exit 1"]));
crate::baseline_case!(holla_flows_docker_remove_gate1_120x40_truecolor => Case::new("holla/flows/docker/remove_gate1/120x40/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Stop and remove all", "enter", "wait:gate 1 of 2"]));
crate::baseline_case!(holla_flows_docker_remove_gate2_120x40_truecolor => Case::new("holla/flows/docker/remove_gate2/120x40/truecolor", HOLLA, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Stop and remove all", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type REMOVE ALL CONTAINERS ON mbp"]));
// h_hp10: the capped services listing and the failing stop verb.
crate::baseline_case!(holla_flows_brew_services_120x40_truecolor => Case::new("holla/flows/brew/services/120x40/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Homebrew services", "enter", "wait:svc02"]));
crate::baseline_case!(holla_flows_brew_services_stop_failed_120x40_truecolor => Case::new("holla/flows/brew/services_stop_failed/120x40/truecolor", HOLLA, &["--scenario", "parity-brew-services", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Stop svc02", "enter", "right", "enter", "wait:Bootstrap failed"]));
// h_hp11/h_hp12: the special-category cleanup reviews (Right opens the
// category fold) and their trash gates.
crate::baseline_case!(holla_flows_gradle_cleanup_120x40_truecolor => Case::new("holla/flows/gradle/cleanup/120x40/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Clean Gradle outputs", "enter", "wait:Cleanup · Gradle outputs", "right", "sleep:300"]));
crate::baseline_case!(holla_flows_gradle_gate1_120x40_truecolor => Case::new("holla/flows/gradle/gate1/120x40/truecolor", HOLLA, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Clean Gradle outputs", "enter", "wait:Cleanup · Gradle outputs", "right", "d", "sleep:300"]));
crate::baseline_case!(holla_flows_idea_cleanup_120x40_truecolor => Case::new("holla/flows/idea/cleanup/120x40/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Clean IntelliJ", "enter", "right", "wait:idea.clean"]));
crate::baseline_case!(holla_flows_idea_gate2_120x40_truecolor => Case::new("holla/flows/idea/gate2/120x40/truecolor", HOLLA, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Clean IntelliJ", "enter", "right", "wait:idea.clean", "d", "wait:gate 1 of 2", "right", "enter", "wait:Type TRASH 4 UNDER /Users/alex/work/ide ON mbp"]));
// h_hp13: the brew upgrade batch run to completion, then the everything
// plan over it (the mbp/brew variant — the audit fixture is devbox/mise).
crate::baseline_case!(holla_flows_brew_upgrade_batch_120x40_truecolor => Case::new("holla/flows/brew/upgrade_batch/120x40/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Upgrade Homebrew packages", "enter", "right", "enter", "wait:Outdated Formulae · 2"]).timeout(15_000));
crate::baseline_case!(holla_flows_brew_upgrade_plan_120x40_truecolor => Case::new("holla/flows/brew/upgrade_plan/120x40/truecolor", HOLLA, &["--scenario", "parity-upgrade-managers", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Upgrade Homebrew packages", "enter", "right", "enter", "wait:Outdated Formulae · 2", "alt-0", "ctrl-u", "type:Upgrade everything", "enter", "wait:9 steps"]).timeout(15_000));
// h_hp15 — task_input prompt / input_mode / answered / stopping — DROP
// (4): "waiting for input" and "cancelling" keep the activity spinner and
// the `running · N s` duration advancing every tick (animating() stays
// true), so wait_stable(400ms) can never hold and no paused frame reaches
// them (ticks must advance the script). The terminal states below are
// stable and captured instead. Same class as the §4 wall-clock drops.
crate::baseline_case!(holla_flows_task_input_cancelled_120x40_truecolor => Case::new("holla/flows/task/input_cancelled/120x40/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Deploy the release", "enter", "right", "enter", "wait:Password:", "i", "type:hunter2", "enter", "wait:Proceed with rollout?", "type:n", "enter", "wait:rollout cancelled by operator"]).timeout(15_000));
crate::baseline_case!(holla_flows_task_input_killed_120x40_truecolor => Case::new("holla/flows/task/input_killed/120x40/truecolor", HOLLA, &["--scenario", "parity-task-input", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Deploy the release", "enter", "right", "enter", "wait:Password:", "i", "type:hunter2", "enter", "wait:Proceed with rollout?", "type:n", "enter", "wait:rollout cancelled by operator", "alt-0", "ctrl-u", "type:Run the stubborn worker", "enter", "wait:ignoring SIGTERM", "s", "wait:killed (SIGKILL)"]).timeout(15_000));
// h_hp17 trust — covered by holla/fade/trust_body_wheel (same Trust page,
// captured with scroll-fade evidence; an unscrolled twin would double-cover
// the surface). h_hp17 config — covered by holla/flows/config_page (same
// SnapshotPage "config:" class; different fixture/trust state only).
// h_hp17 trusted_running is NOT the trust_accepted twin: the action has no
// simulated outcome, so approval runs it and it fails at exit 127.
crate::baseline_case!(holla_flows_trusted_deploy_failed_120x40_truecolor => Case::new("holla/flows/trust/deploy_failed/120x40/truecolor", HOLLA, &["--scenario", "parity-custom-actions", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Deploy preview", "enter", "wait:Trust ~/work/team/.holla.toml?", "right", "enter", "wait:exit 127"]));
// h_hp18 scanning / cancelled — DROP (2): the mid-scan percentage, spinner
// glyph and the cancel percentage are pure functions of the wall-clock tick
// count (scan_ticks=48, parity.rs; the scan starts on Enter). Paused can
// only pin scan-start (0%), reduced lands on a different phase every run.
// The completed scan is the stable boundary and is captured.
crate::baseline_case!(holla_flows_disk_scan_complete_120x40_truecolor => Case::new("holla/flows/disk/scan_complete/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-scan", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:2 unreadable"]).timeout(15_000));
// h_hp19: the finished tree, apparent sort, noise unfolded, and a
// selection (top_files is already captured as holla/flows/disk_top_files).
crate::baseline_case!(holla_flows_disk_tree_120x40_truecolor => Case::new("holla/flows/disk/tree/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete"]).timeout(15_000));
crate::baseline_case!(holla_flows_disk_tree_apparent_120x40_truecolor => Case::new("holla/flows/disk/tree_apparent/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "wait:largest first · apparent"]).timeout(15_000));
crate::baseline_case!(holla_flows_disk_tree_unfolded_120x40_truecolor => Case::new("holla/flows/disk/tree_unfolded/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "s", "down", "down", "f", "wait:allocated · unfolded"]).timeout(15_000));
crate::baseline_case!(holla_flows_disk_tree_selected_120x40_truecolor => Case::new("holla/flows/disk/tree_selected/120x40/truecolor", HOLLA, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "s", "down", "down", "f", "f", "space", "wait:1 selected (176.0"]).timeout(15_000));
// h_hp20: the categories page (the fade twin captures it scrolled),
// DerivedData opened, and the cursor jumped to the last category.
crate::baseline_case!(holla_flows_cleanup_categories_120x40_truecolor => Case::new("holla/flows/cleanup/categories/120x40/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories"]));
crate::baseline_case!(holla_flows_cleanup_derived_data_120x40_truecolor => Case::new("holla/flows/cleanup/derived_data/120x40/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories", "right", "wait:▾ Xcode DerivedData"]));
crate::baseline_case!(holla_flows_cleanup_artifacts_120x40_truecolor => Case::new("holla/flows/cleanup/artifacts/120x40/truecolor", HOLLA, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories", "end", "wait:Project artifacts"]));
// h_hp21: the delete-safety trash gates (the report is already captured as
// holla/flows/cleanup_report; the gate facts carry no live-age rows, so
// reduced is safe once the scan completes).
crate::baseline_case!(holla_flows_cleanup_gate1_120x40_truecolor => Case::new("holla/flows/cleanup/gate1/120x40/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2"]).timeout(15_000));
crate::baseline_case!(holla_flows_cleanup_gate2_120x40_truecolor => Case::new("holla/flows/cleanup/gate2/120x40/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2"]).timeout(15_000));
crate::baseline_case!(holla_flows_cleanup_gate2_typed_120x40_truecolor => Case::new("holla/flows/cleanup/gate2_typed/120x40/truecolor", HOLLA, &["--scenario", "parity-delete-safety", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:TRASH 1 UNDER /Users/alex/Projects/safe ON mbp"]).timeout(15_000));
// h_hp22: the cleanup history page (one prior record at boot).
crate::baseline_case!(holla_flows_cleanup_history_120x40_truecolor => Case::new("holla/flows/cleanup/history/120x40/truecolor", HOLLA, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Show cleanup history", "enter", "wait:operation log records"]));
// h_hp23: Linux without a Trash backend (the gate says every item will
// fail, never fall back) and the macOS Spotlight timeout.
crate::baseline_case!(holla_flows_platforms_linux_gate1_120x40_truecolor => Case::new("holla/flows/platforms/linux_gate1/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend"]));
crate::baseline_case!(holla_flows_platforms_linux_gate2_120x40_truecolor => Case::new("holla/flows/platforms/linux_gate2/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend", "right", "enter", "wait:Type TRASH 1 UNDER /home/alex ON devbox"]));
// h_hp23_linux_report: commit the linux trash (no backend) — every item
// fails, never falls back. Reduced: the cleanup job is tick-driven.
crate::baseline_case!(holla_flows_cleanup_linux_report_120x40_truecolor => Case::new("holla/flows/cleanup/linux_report/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms-linux", "--motion", "reduced"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend", "right", "enter", "wait:Type TRASH 1 UNDER /home/alex ON devbox", "enter", "type:TRASH 1 UNDER /home/alex ON devbox", "enter", "tab", "enter", "wait:Cleanup report"]).timeout(15_000));
crate::baseline_case!(holla_flows_mac_top_files_120x40_truecolor => Case::new("holla/flows/platforms/files/120x40/truecolor", HOLLA, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["type:Top files on this Mac", "enter", "wait:did not finish within 5 s"]));
// h_p3_docker_drift: the first Execute meets the drift and bounces back to
// gate 1 with the Changed row (the docker_done chain passes through this
// state; all key-driven, so paused pins the gate-1 Provenance label).
crate::baseline_case!(holla_flows_docker_drift_120x40_truecolor => Case::new("holla/flows/docker/drift/120x40/truecolor", HOLLA, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:a new container"]));

// --------------------------------------------------------------------- fade --
// Wheel scroll-fade over live sessions, centrally expanded by the shared
// canonical live matrix.

/// First occurrence of `needle` as `(row, col)`, waiting until it appears.
fn find(s: &mut Session, needle: &str) -> (u16, u16) {
    let mut hit = None;
    s.wait_until(|screen| {
        hit = screen.find(needle);
        hit.is_some()
    })
    .unwrap_or_else(|e| panic!("`{needle}` never appeared: {e:#}"));
    hit.expect("wait_until passed with the needle on screen")
}

/// `notches` wheel-down steps over `needle`'s cell, paced like a send step.
fn wheel_down(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        s.scroll(col, row, Scroll::Down).expect("wheel scroll");
        std::thread::sleep(Duration::from_millis(120));
    }
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_fade_trust_body_wheel_matrix() {
    // The trust definition is the scrollable body (review.rs TRUST_BODY);
    // two notches put the fade at its top edge.
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
    // The 18 insight categories overflow the list at 120x40; the wheel
    // target is a row that appears only in the list (never in the detail
    // panel, which mirrors the selected category).
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
        // Rows are clickable children and shadow the list's scroll region in
        // hit_scroll's topmost-wins lookup, so the wheel lands on the blank
        // line below the needle (no child region there) instead.
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
    // 4500 lines in sixteen ticks: the retention panel states the drop and
    // the tail sits at the bottom; after the burst nothing animates, so the
    // settle captures the exact end frame (h_fade_burst's successor).
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
