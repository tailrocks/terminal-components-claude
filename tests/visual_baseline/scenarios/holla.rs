//! holla captures (253): 9 concept scenarios under `holla/concept/`, 23 parity
//! scenarios under `holla/parity/`, journeys under `holla/flows/` (including
//! the H1-H21 chord/gate/journey additions), wheel scroll-fade under
//! `holla/fade/`. The two audit fixtures (rust-dirty, upgrade-plan) keep
//! statics only in the audit rows below (`holla/audit/`, 5x5 matrix - the dedupe rule).
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

use super::{Bin, Root, Variant, interact};
use crate::support::Color;

#[rustfmt::skip]
pub const SURFACES: &[Root] = &[
    // ---------------------------------------------------------------- concept --
    Root::new("holla/concept/first-use", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(72, 20, Color::Truecolor, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/concept/monorepo-root", Bin::Holla, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/concept/monorepo-child", Bin::Holla, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/concept/docker-cleanup", Bin::Holla, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    // The 16-colour probe proved this boot stream can outlive the default at
    // exactly 100x30/16; retain that timeout only for that exact combo.
    Root::new("holla/concept/disk-cleanup", Bin::Holla, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Ansi256, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Ansi16, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], "holla❯").timeout(20_000),
    ]),
    // activities-multi's fast-forward (5 live activities × 40 ticks) is the
    // slowest boot in the suite; under the full-suite 8-thread load the default
    // 8 s boot budget can expire before the first frame (empty-screen timeouts).
    // A 20 s boot budget — the pixel gate is untouched.
    Root::new("holla/concept/activities-multi", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000).variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
        Variant::new(100, 30, Color::None, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
    ]),
    Root::new("holla/concept/remote-host", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/concept/launch-failure", Bin::Holla, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/concept/hard-cases", Bin::Holla, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Ansi256, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(72, 20, Color::Truecolor, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    // concept extras (verbatim remap of the ad-hoc legacy combos)

    // ----------------------------------------------------------------- parity --
    Root::new("holla/parity/discovery", Bin::Holla, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/history", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/files", Bin::Holla, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-files", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/browser", Bin::Holla, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-browser", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/git-current", Bin::Holla, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/git-batch", Bin::Holla, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-git-batch", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/task-sources", Bin::Holla, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/cargo", Bin::Holla, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/docker", Bin::Holla, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/brew-services", Bin::Holla, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/gradle", Bin::Holla, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/idea", Bin::Holla, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/upgrade-managers", Bin::Holla, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-upgrade-managers", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/executor", Bin::Holla, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-executor", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/task-input", Bin::Holla, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-task-input", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/custom-actions", Bin::Holla, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-custom-actions", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/disk-scan", Bin::Holla, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-disk-scan", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/disk-navigation", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-disk-navigation", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/insights", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/delete-safety", Bin::Holla, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-delete-safety", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/cleanup-results", Bin::Holla, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/platforms", Bin::Holla, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    Root::new("holla/parity/platforms-linux", Bin::Holla, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯"),
    ]),
    // ------------------------------------------------------------------ audit --
    Root::new("holla/audit/rust", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯"),
    Root::new("holla/audit/upgrade", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "paused", "--frame", "40"], "holla❯"),
];

#[rustfmt::skip]
pub const ELEMENTS: &[Root] = &[
    // Pending coverage (plan §C2): hover, held press (paused), selected+hover.
    Root::new("holla/elements/menubar/file-hover", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").hover("File").pending(),
    Root::new("holla/elements/menubar/file-pressed", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").press("File").pending(),
    Root::new("holla/elements/results/row-hover", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").hover("Open btm").pending(),
    Root::new("holla/elements/results/row-pressed", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").press("Open btm").pending(),
    Root::new("holla/elements/results/selected-hover", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").hover("› Show system resources").pending(),
    Root::new("holla/elements/explore/tab-hover", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").hover("Disk").pending(),
];

#[rustfmt::skip]
pub const OVERLAYS: &[Root] = &[
    // Open overlays captured by existing journeys (names unchanged), with
    // pending dismissal scenarios (`-dismissed-esc`, `-dismissed-outside`).
    Root::new("holla/flows/help/overlay", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f1"]).dismiss_pending(),
    Root::new("holla/flows/activities/overlay", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-g"]).dismiss_pending(),
    // H3/H4: F10 opens the File menu, Right crosses to Go (the
    // app_tests_flows.rs menu proof, PTY form).
    Root::new("holla/flows/menu/file", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f10", "wait:Alternatives…"]).dismiss_pending(),
    Root::new("holla/flows/menu/go", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f10", "right", "wait:Parent scope"]).dismiss_pending(),
    // H15: Ctrl+Q with live activities opens the quit confirmation.
    Root::new("holla/flows/quit/confirm", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-q", "wait:still running"]).dismiss_pending(),
    Root::new("holla/flows/git/batch_picker", Bin::Holla, &["--scenario", "parity-git-batch", "--motion", "reduced"], "holla❯").sends(&["type:Pull 4 sibling", "enter", "right", "enter", "wait:Fast-forward", "wait:2 ok · 2 failed", "ctrl-g", "wait:0 live"]).timeout(15_000).dismiss_pending(),
    Root::new("holla/flows/cargo/clean_confirm", Bin::Holla, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:cargo clean", "enter", "wait:Esc Cancel"]).dismiss_pending(),
    Root::new("holla/flows/upgrade/confirm", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "reduced"], "holla❯").sends(&["down", "down", "down", "down", "down", "space", "c"]).dismiss_pending(),
    // No dismissal rows: under reduced motion the screen behind the prompt
    // keeps animating, so a dismissed frame never settles.
    Root::new("holla/flows/trust/prompt", Bin::Holla, &["--scenario", "monorepo-child", "--motion", "reduced"], "holla❯").sends(&["type:test", "enter"]),
    // Pending: pointer-opened menus, with both dismissals.
    Root::new("holla/overlays/file_menu_click", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").click("File").pending().dismiss_pending(),
    Root::new("holla/overlays/results_context", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").right_click("Open btm").pending().dismiss_pending(),
];

#[rustfmt::skip]
pub const JOURNEYS: &[Root] = &[
    // ------------------------------------------------------------------ flows --

    // paused, not reduced like the other journeys: parity-history renders a
    // wall-clock relative-time label (`live · N s ago`) that reduced-motion ticks
    // advance — the pinned frame keeps it deterministic (same contract as
    // finder_query-selected below).
    Root::new("holla/flows/finder/query", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:pull"]),
    Root::new("holla/flows/finder/query-selected", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:pull", "ctrl-a"]),
    Root::new("holla/flows/files/results", Bin::Holla, &["--scenario", "parity-files", "--motion", "reduced"], "holla❯").sends(&["type:Find files under home", "enter", "type:readme"]),
    Root::new("holla/flows/files/unicode", Bin::Holla, &["--scenario", "parity-files", "--motion", "reduced"], "holla❯").sends(&["type:Find files under home", "enter", "ctrl-u", "type:café"]),
    Root::new("holla/flows/browser/hidden", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "ctrl-h"]),
    Root::new("holla/flows/browser/preview", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "down", "down", "down"]),
    // h_hp04_preview_control: same listing as browser_preview (big.log at 3 downs);
    // control.txt is 7 downs (dirs first). Preview sanitises ESC/BEL to �.
    Root::new("holla/flows/files/preview_control", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "down", "down", "down", "down", "down", "down", "down", "sleep:300"]),
    Root::new("holla/flows/cleanup/plan", Bin::Holla, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], "holla❯").sends(&["c"]),
    Root::new("holla/flows/cleanup/gate-1", Bin::Holla, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], "holla❯").sends(&["c", "c"]),
    Root::new("holla/flows/upgrade/excluded", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "reduced"], "holla❯").sends(&["down", "down", "down", "down", "down", "space"]),
    Root::new("holla/flows/remote/gate-1", Bin::Holla, &["--scenario", "remote-host", "--motion", "reduced"], "holla❯").sends(&["type:restart payments", "enter"]),
    // ------------------------------------------------------- flows (new, H1-H21) --

    // H1: Alt+Enter alternatives menu. `alt-enter` is not chord-expressible, so
    // the bytes a real terminal sends for it are typed verbatim in one write
    // (`\x1b\r`, the pointer.rs verbatim technique). Paused: the preview panel
    // behind the menu carries a `live · N s ago` label.
    Root::new("holla/flows/alternatives", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:\u{1b}\r", "wait:Pin here"]),
    // H2: Alt+1 jumps to the second tab (frontend dev); the strip's spinner
    // glyphs are tick frames, so the clock stays frozen.
    Root::new("holla/flows/activities/jump", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["alt-1"]),
    // H5/H6: the two-gate remote action. Gate 1 continues with Right+Enter (its
    // `y` copies the sequence, review.rs) — the spec's `y` was a misread. Paused:
    // the gate-1 facts render `live · N s ago`.
    Root::new("holla/flows/remote/gate2", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type RESTART PAYMENTS ON prod-eu-1"]),
    // H6: Enter arms the ack field for editing, then the host-bound phrase is
    // typed; the dialog stays open with the armed token visible.
    Root::new("holla/flows/remote/gate2_typed", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type RESTART PAYMENTS ON prod-eu-1", "enter", "type:RESTART PAYMENTS ON prod-eu-1"]),
    // H7-H9: scope chords from the monorepo root (the scope walk of
    // app_tests_flows.rs). Key-driven, and paused pins the preview panel's
    // `live · N s ago` freshness label (the finder_query lesson; the
    // system-scope preview demonstrably carries it).
    Root::new("holla/flows/scope/parent", Bin::Holla, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-up", "wait:No project root above"]),
    Root::new("holla/flows/scope/children", Bin::Holla, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-down", "wait:Children · 3 projects"]),
    Root::new("holla/flows/scope/system", Bin::Holla, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-up", "ctrl-up", "wait:scope ‹ system ›"]),
    // H10/H11: query undo/redo on the same paused fixture as finder_query (the
    // history preview's `live · N s ago` froze that pair too). Undo is
    // per-keystroke: emptying `pull` takes four Ctrl+Z, restoring it four
    // Ctrl+Y.
    Root::new("holla/flows/query/undone", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:pull", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z"]),
    Root::new("holla/flows/query/redone", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:pull", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-y", "ctrl-y", "ctrl-y", "ctrl-y"]),
    // H12/H13: the Go-to-path picker and its validation error. Reduced: the
    // files page's pending listing only applies on a tick (paused would wedge it
    // animating); the picker and the error carry no tick label.
    Root::new("holla/flows/files/jump", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path"]),
    Root::new("holla/flows/files/jump_error", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path", "type:nope", "enter", "wait:no such file or directory"]),
    // H14: Ctrl+G with nothing started (the h_flow_activities_empty state, on
    // the spec's first-use fixture).
    Root::new("holla/flows/activities/empty", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-g", "wait:Nothing has been started yet"]),
    // H16: trust accepted (Cancel is the default focus; Right arms Trust). The
    // accept runs the task, which lands on its arguments page — the post-accept
    // state; the `Trusted …` status line outlives the settle (5 virtual seconds).
    Root::new("holla/flows/trust/accepted", Bin::Holla, &["--scenario", "monorepo-child", "--motion", "reduced"], "holla❯").sends(&["type:test", "enter", "wait:Trust ~/work/acme/apps/frontend/mise.toml?", "right", "enter", "wait:Trusted ~/work/acme/apps/frontend/mise.toml"]),
    // H17: an item with structured args opens the ArgsPage instead of running
    // (`port` ranks the port action first — the catalog's exact-keyword proof).
    Root::new("holla/flows/args/page", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "reduced"], "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port"]),
    // h_flow_args: clone-repo Arguments page (`clone` ranks github.clone first).
    // rust-dirty (not monorepo-child): destination ~/work/holla, owner alex-dev.
    Root::new("holla/flows/args/clone", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "reduced"], "holla❯").sends(&["type:clone", "enter", "wait:Clone a GitHub repository"]),
    // H18: docker gate 2 with the target-bound phrase typed (h_p3_docker's
    // gate2_typed state). All key-driven; paused keeps the gate-1 `Provenance ·
    // N s ago` label frozen.
    Root::new("holla/flows/docker/gate2_typed", Bin::Holla, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox"]),
    // H19: the full run. The first Execute meets the plan drift and bounces back
    // to gate 1 (the docker_cleanup_takes_two_gates flow); the second pass
    // executes and ticks drive the plan to Done. Reduced is required (the run is
    // tick-driven); the end frame is stable because nothing animates after Done.
    Root::new("holla/flows/docker/done", Bin::Holla, &["--scenario", "docker-cleanup", "--motion", "reduced"], "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:Plan changed", "right", "enter", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:7 succeeded"]).timeout(60_000),
    // H20: the Spotlight top-files page over the finished scan (`t` opens it,
    // disk.rs). Reduced: the tree scan completes only through ticks.
    Root::new("holla/flows/disk/top_files", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "t", "wait:Top files · Spotlight"]).timeout(15_000),
    // H21: a cleanup report. The spec's `down…, enter` was a misread: the
    // history page has no reports at boot (nothing links one), so Tab fell
    // through to the menu bar. The real route is the hp22 flow: select `big`
    // (the largest child, one row under the root) in the finished scan, `d`,
    // both gates, and the commit pushes the report page — one tick later it is
    // the finished Cleanup report. Reduced: the scan and the cleanup job are
    // tick-driven; the page's `· live ·` meta is a constant ("1 s ago"), not a
    // tick readout. `space` selects the row first — `d` refuses an empty
    // selection ("Select at least one entry first").
    Root::new("holla/flows/cleanup/report", Bin::Holla, &["--scenario", "parity-cleanup-results", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:TRASH 1 UNDER /Users/alex/Projects/safe ON mbp", "enter", "tab", "enter", "wait:Cleanup report"]).timeout(15_000),
    // ------------------------------------------- coverage holes (legacy h_flow_*,
    // h_hp*, h_p3 states; keys from the retired tools/holla_flows.sh and
    // tools/holla_parity_flows.sh, motion hardened per the determinism playbook) --

    // h_flow_child_query: the `test` query in the monorepo child (the
    // trust_prompt journey's pre-Enter state).
    Root::new("holla/flows/child/query", Bin::Holla, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:test", "wait:Results · 17"]),
    // h_flow_system: a query with no matches leaves an empty preview; the
    // `Cancelled` status comes from leaving gate 1 (the legacy chain typed
    // `blocking`, opened the blocker page, then appended `resources`).
    Root::new("holla/flows/query/empty_preview", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "escape", "escape", "type:blocking", "enter", "wait:Who is blocking", "escape", "type:resources", "wait:No matches for"]),
    // h_flow_remote_query: the remote query typed, not yet run.
    Root::new("holla/flows/remote/query", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments"]),
    // h_flow_port_snapshot: the args flow's end state — Ctrl+S takes the
    // defaults (port 5173) and opens the listeners snapshot.
    Root::new("holla/flows/args", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port", "ctrl-s", "wait:Port 5173"]),
    // h_flow_upgrade_80_facts: the plan step-facts drawer at the minimum size
    // (`p` toggles it, plan.rs).
    Root::new("holla/flows/upgrade/facts", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["p", "wait:Preflight"]),
    // h_hp01_config_page: the custom-action configuration diagnostics page.
    Root::new("holla/flows/config/page", Bin::Holla, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Custom action configuration", "enter", "wait:sha256:"]),
    // h_hp02_recent — DROP: the usage-populated Recent list ("used N times
    // here") exists only with the history store; the suite sets
    // HOLLA_NO_HISTORY=1 by design (support.rs opts_for — ambient-learning
    // hygiene), so every capture shows "nothing used here yet". The populated
    // state is unreachable by construction, not by nondeterminism.
    // h_hp03_actions: Enter on a find result opens its actions popup.
    Root::new("holla/flows/files/actions", Bin::Holla, &["--scenario", "parity-files", "--motion", "reduced"], "holla❯").sends(&["type:Find files under home", "enter", "wait:of 29 indexed", "ctrl-u", "type:todo", "wait:1 result", "enter", "wait:Open in the OS"]),
    // h_hp04_jumped: the browser after a successful Go-to-path jump.
    Root::new("holla/flows/files/jumped", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:Go to path", "type:~/work", "enter", "wait:Jumped to ~/work"]),
    // h_hp05: git-current — the blocked pull preview, the failed merge and the
    // rejected push (push_rejected keeps the merge's failed tab, like the
    // legacy single-session chain).
    Root::new("holla/flows/git/pull_blocked", Bin::Holla, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Pull", "wait:blocked · 1 modified"]),
    Root::new("holla/flows/git/merge", Bin::Holla, &["--scenario", "parity-git-current", "--motion", "reduced"], "holla❯").sends(&["type:Pull with merge", "enter", "right", "enter", "wait:error: Your local changes"]),
    Root::new("holla/flows/git/push_rejected", Bin::Holla, &["--scenario", "parity-git-current", "--motion", "reduced"], "holla❯").sends(&["type:Pull with merge", "enter", "right", "enter", "wait:error: Your local changes", "alt-0", "ctrl-u", "type:Push", "enter", "right", "enter", "wait:rejected"]),
    // h_hp06: the sibling batch run to completion, then the activities picker
    // over it (2 succeeded, 2 failed).
    Root::new("holla/flows/git/batch", Bin::Holla, &["--scenario", "parity-git-batch", "--motion", "reduced"], "holla❯").sends(&["type:Pull 4 sibling", "enter", "right", "enter", "wait:Fast-forward"]).timeout(15_000),
    // h_hp07: the capped script list and the discovery-diagnostic row.
    Root::new("holla/flows/task/sources", Bin::Holla, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:yarn", "wait:yarn s00"]),
    Root::new("holla/flows/task/sources_diagnostic", Bin::Holla, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Taskfile", "wait:discovery diagnostic"]),
    // h_hp08: the cargo clean row and its one-confirmation dialog.
    Root::new("holla/flows/cargo/clean", Bin::Holla, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:cargo clean", "wait:900.0 MiB"]),
    // h_hp09: the daemon-stage stop-all failure, then the remove-all gates.
    // The gates are taken from a fresh boot under paused: the legacy chain
    // reached them over the failed tab, but that prefix leaves the gate's
    // `Data · live N s ago` re-resolve straddling a second boundary under
    // full-suite load (0/1 s flip-flop, observed). Paused pins the label (and
    // renders "2 s ago" like the legacy frame); the failure itself is covered
    // by docker_remove_failed.
    Root::new("holla/flows/docker/remove_failed", Bin::Holla, &["--scenario", "parity-docker", "--motion", "reduced"], "holla❯").sends(&["type:Stop all containers", "enter", "right", "enter", "wait:exit 1"]),
    Root::new("holla/flows/docker/remove_gate1", Bin::Holla, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Stop and remove all", "enter", "wait:gate 1 of 2"]),
    Root::new("holla/flows/docker/remove_gate2", Bin::Holla, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Stop and remove all", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type REMOVE ALL CONTAINERS ON mbp"]),
    // h_hp10: the capped services listing and the failing stop verb.
    Root::new("holla/flows/brew/services", Bin::Holla, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Homebrew services", "enter", "wait:svc02"]),
    Root::new("holla/flows/brew/services_stop_failed", Bin::Holla, &["--scenario", "parity-brew-services", "--motion", "reduced"], "holla❯").sends(&["type:Stop svc02", "enter", "right", "enter", "wait:Bootstrap failed"]),
    // h_hp11/h_hp12: the special-category cleanup reviews (Right opens the
    // category fold) and their trash gates.
    Root::new("holla/flows/gradle/cleanup", Bin::Holla, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Clean Gradle outputs", "enter", "wait:Cleanup · Gradle outputs", "right", "sleep:300"]),
    Root::new("holla/flows/gradle/gate1", Bin::Holla, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Clean Gradle outputs", "enter", "wait:Cleanup · Gradle outputs", "right", "d", "sleep:300"]),
    Root::new("holla/flows/idea/cleanup", Bin::Holla, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Clean IntelliJ", "enter", "right", "wait:idea.clean"]),
    Root::new("holla/flows/idea/gate2", Bin::Holla, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Clean IntelliJ", "enter", "right", "wait:idea.clean", "d", "wait:gate 1 of 2", "right", "enter", "wait:Type TRASH 4 UNDER /Users/alex/work/ide ON mbp"]),
    // h_hp13: the brew upgrade batch run to completion, then the everything
    // plan over it (the mbp/brew variant — the audit fixture is devbox/mise).
    Root::new("holla/flows/brew/upgrade_batch", Bin::Holla, &["--scenario", "parity-upgrade-managers", "--motion", "reduced"], "holla❯").sends(&["type:Upgrade Homebrew packages", "enter", "right", "enter", "wait:Outdated Formulae · 2"]).timeout(15_000),
    Root::new("holla/flows/brew/upgrade_plan", Bin::Holla, &["--scenario", "parity-upgrade-managers", "--motion", "reduced"], "holla❯").sends(&["type:Upgrade Homebrew packages", "enter", "right", "enter", "wait:Outdated Formulae · 2", "alt-0", "ctrl-u", "type:Upgrade everything", "enter", "wait:9 steps"]).timeout(15_000),
    // h_hp15 — task_input prompt / input_mode / answered / stopping — DROP
    // (4): "waiting for input" and "cancelling" keep the activity spinner and
    // the `running · N s` duration advancing every tick (animating() stays
    // true), so wait_stable(400ms) can never hold and no paused frame reaches
    // them (ticks must advance the script). The terminal states below are
    // stable and captured instead. Same class as the §4 wall-clock drops.
    Root::new("holla/flows/task/input_cancelled", Bin::Holla, &["--scenario", "parity-task-input", "--motion", "reduced"], "holla❯").sends(&["type:Deploy the release", "enter", "right", "enter", "wait:Password:", "i", "type:hunter2", "enter", "wait:Proceed with rollout?", "type:n", "enter", "wait:rollout cancelled by operator"]).timeout(15_000),
    Root::new("holla/flows/task/input_killed", Bin::Holla, &["--scenario", "parity-task-input", "--motion", "reduced"], "holla❯").sends(&["type:Deploy the release", "enter", "right", "enter", "wait:Password:", "i", "type:hunter2", "enter", "wait:Proceed with rollout?", "type:n", "enter", "wait:rollout cancelled by operator", "alt-0", "ctrl-u", "type:Run the stubborn worker", "enter", "wait:ignoring SIGTERM", "s", "wait:killed (SIGKILL)"]).timeout(15_000),
    // h_hp17 trust — covered by holla/fade/trust_body_wheel (same Trust page,
    // captured with scroll-fade evidence; an unscrolled twin would double-cover
    // the surface). h_hp17 config — covered by holla/flows/config_page (same
    // SnapshotPage "config:" class; different fixture/trust state only).
    // h_hp17 trusted_running is NOT the trust_accepted twin: the action has no
    // simulated outcome, so approval runs it and it fails at exit 127.
    Root::new("holla/flows/trust/deploy_failed", Bin::Holla, &["--scenario", "parity-custom-actions", "--motion", "reduced"], "holla❯").sends(&["type:Deploy preview", "enter", "wait:Trust ~/work/team/.holla.toml?", "right", "enter", "wait:exit 127"]),
    // h_hp18 scanning / cancelled — DROP (2): the mid-scan percentage, spinner
    // glyph and the cancel percentage are pure functions of the wall-clock tick
    // count (scan_ticks=48, parity.rs; the scan starts on Enter). Paused can
    // only pin scan-start (0%), reduced lands on a different phase every run.
    // The completed scan is the stable boundary and is captured.
    Root::new("holla/flows/disk/scan_complete", Bin::Holla, &["--scenario", "parity-disk-scan", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:2 unreadable"]).timeout(15_000),
    // h_hp19: the finished tree, apparent sort, noise unfolded, and a
    // selection (top_files is already captured as holla/flows/disk_top_files).
    Root::new("holla/flows/disk/tree", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete"]).timeout(15_000),
    Root::new("holla/flows/disk/tree_apparent", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "wait:largest first · apparent"]).timeout(15_000),
    Root::new("holla/flows/disk/tree_unfolded", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "s", "down", "down", "f", "wait:allocated · unfolded"]).timeout(15_000),
    Root::new("holla/flows/disk/tree_selected", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "s", "down", "down", "f", "f", "space", "wait:1 selected (176.0"]).timeout(15_000),
    // h_hp20: the categories page (the fade twin captures it scrolled),
    // DerivedData opened, and the cursor jumped to the last category.
    Root::new("holla/flows/cleanup/categories", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories"]),
    Root::new("holla/flows/cleanup/derived_data", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories", "right", "wait:▾ Xcode DerivedData"]),
    Root::new("holla/flows/cleanup/artifacts", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories", "end", "wait:Project artifacts"]),
    // h_hp21: the delete-safety trash gates (the report is already captured as
    // holla/flows/cleanup_report; the gate facts carry no live-age rows, so
    // reduced is safe once the scan completes).
    Root::new("holla/flows/cleanup/gate1", Bin::Holla, &["--scenario", "parity-delete-safety", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2"]).timeout(15_000),
    Root::new("holla/flows/cleanup/gate2", Bin::Holla, &["--scenario", "parity-delete-safety", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2"]).timeout(15_000),
    Root::new("holla/flows/cleanup/gate2_typed", Bin::Holla, &["--scenario", "parity-delete-safety", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:TRASH 1 UNDER /Users/alex/Projects/safe ON mbp"]).timeout(15_000),
    // h_hp22: the cleanup history page (one prior record at boot).
    Root::new("holla/flows/cleanup/history", Bin::Holla, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Show cleanup history", "enter", "wait:operation log records"]),
    // h_hp23: Linux without a Trash backend (the gate says every item will
    // fail, never fall back) and the macOS Spotlight timeout.
    Root::new("holla/flows/platforms/linux_gate1", Bin::Holla, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend"]),
    Root::new("holla/flows/platforms/linux_gate2", Bin::Holla, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend", "right", "enter", "wait:Type TRASH 1 UNDER /home/alex ON devbox"]),
    // h_hp23_linux_report: commit the linux trash (no backend) — every item
    // fails, never falls back. Reduced: the cleanup job is tick-driven.
    Root::new("holla/flows/cleanup/linux_report", Bin::Holla, &["--scenario", "parity-platforms-linux", "--motion", "reduced"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend", "right", "enter", "wait:Type TRASH 1 UNDER /home/alex ON devbox", "enter", "type:TRASH 1 UNDER /home/alex ON devbox", "enter", "tab", "enter", "wait:Cleanup report"]).timeout(15_000),
    Root::new("holla/flows/platforms/files", Bin::Holla, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Top files on this Mac", "enter", "wait:did not finish within 5 s"]),
    // h_p3_docker_drift: the first Execute meets the drift and bounces back to
    // gate 1 with the Changed row (the docker_done chain passes through this
    // state; all key-driven, so paused pins the gate-1 Provenance label).
    Root::new("holla/flows/docker/drift", Bin::Holla, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "tab", "enter", "wait:a new container"]),
    // --------------------------------------------------------------------- fade --
    // Wheel scroll-fade over live sessions. Below 100 columns, Home removes
    // width-dependent cursor drift and Right opens the compact preview drawer.
    Root::new("holla/fade/browser_wheel", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "down", "down", "down", "wait:first 2000 lines"]).compact(100, &["type:Browse ~/work/site", "enter", "wait:16 entries", "home", "down", "down", "down", "right", "wait:first 2000 lines"]).live(interact::browser_wheel),
    Root::new("holla/fade/trust_body_wheel", Bin::Holla, &["--scenario", "parity-custom-actions", "--motion", "reduced"], "holla❯").sends(&["type:Deploy preview", "enter", "wait:Trust ~/work/team/.holla.toml?"]).live(interact::trust_body_wheel),
    // The 18 insight categories overflow the list at 120x40.
    Root::new("holla/fade/cleanup_list_wheel", Bin::Holla, &["--scenario", "parity-insights", "--motion", "reduced"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:18 categories"]).live(interact::cleanup_list_wheel),
    // 4500 lines in sixteen ticks; after the burst nothing animates.
    Root::new("holla/fade/executor_burst-end", Bin::Holla, &["--scenario", "parity-executor", "--motion", "reduced"], "holla❯").sends(&["type:Emit a burst", "enter", "wait:500 earlier lines dropped"]),
];

#[rustfmt::skip]
pub const RESIZES: &[Root] = &[
    Root::new("holla/resize/rust-dirty_shrunk", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").timeout(30_000).resize((120, 40), interact::resize_case),
    Root::new("holla/resize/rust-dirty_grown", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").timeout(30_000).resize((80, 24), interact::resize_case),
    // Pending: a second fixture across the breakpoints.
    Root::new("holla/resize/first-use_from160x50", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").timeout(30_000).resize((160, 50), interact::resize_case).pending(),
    Root::new("holla/resize/first-use_from72x20", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").timeout(30_000).resize((72, 20), interact::resize_case).pending(),
];
