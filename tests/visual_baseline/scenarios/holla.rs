use super::{Bin, Root, Variant, interact};
use crate::support::Color;

const HISTORY_ON: &[(&str, &str)] = &[("HOLLA_NO_HISTORY", "0")];

#[rustfmt::skip]
pub const SURFACES: &[Root] = &[

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


    Root::new("holla/concept/disk-cleanup", Bin::Holla, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯").variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::None, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Ansi256, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "40"], "holla❯"),
        Variant::new(100, 30, Color::Ansi16, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], "holla❯").timeout(20_000),
    ]),




    Root::new("holla/concept/activities-multi", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000).variants(&[
        Variant::new(80, 24, Color::Truecolor, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
        Variant::new(100, 30, Color::Truecolor, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
        Variant::new(120, 40, Color::Truecolor, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
        Variant::new(160, 50, Color::Truecolor, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
        Variant::new(100, 30, Color::None, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000),
    ]),





    Root::new("holla/concept/first-use-with-history", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").env(HISTORY_ON),
    Root::new("holla/concept/monorepo-root-with-history", Bin::Holla, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯").env(HISTORY_ON),
    Root::new("holla/concept/activities-multi-with-history", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").timeout(20_000).env(HISTORY_ON),
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

    Root::new("holla/audit/rust", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯"),
    Root::new("holla/audit/upgrade", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "paused", "--frame", "40"], "holla❯"),
];

#[rustfmt::skip]
pub const ELEMENTS: &[Root] = &[

    Root::new("holla/elements/menubar/file-hover", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").hover("File").pending(),
    Root::new("holla/elements/menubar/file-pressed", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").press("File").pending(),
    Root::new("holla/elements/results/row-hover", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").hover("Open btm").pending(),
    Root::new("holla/elements/results/row-pressed", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").press("Open btm").pending(),
    Root::new("holla/elements/results/selected-hover", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").hover("› Show system resources").pending(),

    Root::new("holla/elements/explore/tab-hover", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").hover("Disk").min_size(80, 24).pending(),
];

#[rustfmt::skip]
pub const OVERLAYS: &[Root] = &[


    Root::new("holla/flows/help/overlay", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f1"]).dismiss_pending(),
    Root::new("holla/flows/activities/overlay", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-g"]).dismiss_pending(),


    Root::new("holla/flows/menu/file", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f10", "wait:Copy command"]).dismiss_pending(),
    Root::new("holla/flows/menu/go", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f10", "right", "wait:Parent scope"]).dismiss_pending(),

    Root::new("holla/flows/quit/confirm", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-q", "wait:still running"]).dismiss_pending(),


    Root::new("holla/flows/quit/confirm_ctrl_c", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-c", "wait:still running"]).pending().dismiss_pending(),
    Root::new("holla/flows/quit/confirm_menu", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f10", "wait:Quit", "up", "enter", "wait:still running"]).pending().dismiss_pending(),
    Root::new("holla/flows/git/batch_picker", Bin::Holla, &["--scenario", "parity-git-batch", "--motion", "reduced"], "holla❯").sends(&["type:Pull 4 sibling", "enter", "right", "enter", "wait:Fast-forward", "wait:2 ok · 2 failed", "ctrl-g", "wait:0 live"]).timeout(15_000).dismiss_pending(),
    Root::new("holla/flows/cargo/clean_confirm", Bin::Holla, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:cargo clean", "enter", "wait:Esc Cancel"]).dismiss_pending(),
    Root::new("holla/flows/upgrade/confirm", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "reduced"], "holla❯").sends(&["down", "down", "down", "down", "down", "space", "c"]).dismiss_pending(),



    Root::new("holla/flows/trust/prompt", Bin::Holla, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:test", "enter", "wait:Trust ~/work/acme/apps/frontend/mise.toml?"]).dismiss_pending(),

    Root::new("holla/overlays/file_menu_click", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").click("File").pending().dismiss_pending(),
    Root::new("holla/overlays/results_context", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").right_click("Open btm").pending().dismiss_pending(),


    Root::new("holla/overlays/about", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f10", "right", "right", "wait:About holla", "up", "enter", "wait:Design system"]).pending().dismiss_pending(),

    Root::new("holla/overlays/why", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["f10", "right", "right", "wait:Why is this here?", "up", "up", "enter", "wait:Ranking order"]).pending().dismiss_pending(),

    Root::new("holla/overlays/alias_prompt", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:\u{1b}\r", "wait:Pin here", "up", "up", "up", "up", "enter", "wait:Set an alias"]).pending().dismiss_pending(),

    Root::new("holla/overlays/close_tab", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["alt-1", "ctrl-w", "wait:Keep running"]).pending().dismiss_pending(),

    Root::new("holla/overlays/activity_output_menu", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["alt-1", "shift-f10", "wait:Copy selection"]).pending().dismiss_pending(),
    Root::new("holla/overlays/activity_chip_menu", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["alt-4", "wait:merged logs", "backtab", "shift-f10", "wait:stream"]).pending().dismiss_pending(),
    Root::new("holla/overlays/cleanup_row_menu", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:selected", "shift-f10"]).pending().dismiss_pending(),
    Root::new("holla/overlays/disk_row_menu", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "shift-f10"]).timeout(15_000).pending().dismiss_pending(),
    Root::new("holla/overlays/files_actions", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "shift-f10"]).pending().dismiss_pending(),
    Root::new("holla/overlays/plan_step_menu", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["shift-f10", "wait:step"]).pending().dismiss_pending(),
    Root::new("holla/overlays/review_sequence_menu", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2"]).right_click("sudo systemctl restart").pending().dismiss_pending(),
    Root::new("holla/overlays/snapshot_body_menu", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port", "ctrl-s", "wait:Port 5173", "shift-f10"]).pending().dismiss_pending(),

    Root::new("holla/overlays/plan_cancel", Bin::Holla, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "wait:Plan changed", "right", "enter", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "ctrl-c", "wait:Cancel the remaining steps?"]).timeout(15_000).pending().dismiss_pending(),






    Root::new("holla/flows/alternatives", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:\u{1b}\r", "wait:Pin here"]).dismiss_pending(),


    Root::new("holla/flows/remote/gate2", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type RESTART PAYMENTS ON prod-eu-1"]).dismiss_pending(),


    Root::new("holla/flows/files/jump", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:type a path or a name"]).dismiss_pending(),
];

#[rustfmt::skip]
pub const JOURNEYS: &[Root] = &[






    Root::new("holla/flows/finder/query", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:pull", "wait:▎ pull"]),
    Root::new("holla/flows/finder/query-selected", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:pull", "wait:▎ pull", "ctrl-a"]),
    Root::new("holla/flows/files/results", Bin::Holla, &["--scenario", "parity-files", "--motion", "reduced"], "holla❯").sends(&["type:Find files under home", "enter", "wait:Files › Find", "type:readme", "wait:▎ readme"]),
    Root::new("holla/flows/files/unicode", Bin::Holla, &["--scenario", "parity-files", "--motion", "reduced"], "holla❯").sends(&["type:Find files under home", "enter", "wait:Files › Find", "ctrl-u", "type:café", "wait:▎ café"]),
    Root::new("holla/flows/browser/hidden", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "ctrl-h"]),
    Root::new("holla/flows/browser/preview", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "down", "down", "down"]),


    Root::new("holla/flows/files/preview_control", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "down", "down", "down", "down", "down", "down", "down", "sleep:300"]),
    Root::new("holla/flows/cleanup/plan", Bin::Holla, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], "holla❯").sends(&["c"]),
    Root::new("holla/flows/cleanup/gate-1", Bin::Holla, &["--scenario", "disk-cleanup", "--motion", "paused", "--frame", "80"], "holla❯").sends(&["c", "c"]),
    Root::new("holla/flows/upgrade/excluded", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "reduced"], "holla❯").sends(&["down", "down", "down", "down", "down", "space"]),
    Root::new("holla/flows/remote/gate-1", Bin::Holla, &["--scenario", "remote-host", "--motion", "reduced"], "holla❯").sends(&["type:restart payments", "wait:▎ restart payments", "enter"]),






    Root::new("holla/flows/activities/jump", Bin::Holla, &["--scenario", "activities-multi", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["alt-1"]),





    Root::new("holla/flows/remote/gate2_typed", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type RESTART PAYMENTS ON prod-eu-1", "enter", "type:RESTART PAYMENTS ON prod-eu-1", "wait:▎ RESTART PAYMENTS ON prod-eu-1"]),




    Root::new("holla/flows/scope/parent", Bin::Holla, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-up", "wait:No project root above"]),
    Root::new("holla/flows/scope/children", Bin::Holla, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-down", "wait:‹ children ›"]),
    Root::new("holla/flows/scope/system", Bin::Holla, &["--scenario", "monorepo-root", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-up", "ctrl-up", "wait:scope ‹ system ›"]),




    Root::new("holla/flows/query/undone", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:pull", "wait:▎ pull", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z"]),
    Root::new("holla/flows/query/redone", Bin::Holla, &["--scenario", "parity-history", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:pull", "wait:▎ pull", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z", "ctrl-y", "ctrl-y", "ctrl-y", "ctrl-y"]),



    Root::new("holla/flows/files/jump_error", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:type a path or a name", "type:nope", "enter", "wait:no such file or directory"]),


    Root::new("holla/flows/activities/empty", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["ctrl-g", "wait:Nothing has been started yet"]),



    Root::new("holla/flows/trust/accepted", Bin::Holla, &["--scenario", "monorepo-child", "--motion", "reduced"], "holla❯").sends(&["type:test", "enter", "wait:Trust ~/work/acme/apps/frontend/mise.toml?", "right", "enter", "wait:Trusted ~/work/acme/apps/frontend/mise.toml"]),


    Root::new("holla/flows/args/page", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "reduced"], "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port"]),


    Root::new("holla/flows/args/clone", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "reduced"], "holla❯").sends(&["type:clone", "enter", "wait:Clone a GitHub repository"]),



    Root::new("holla/flows/docker/gate2_typed", Bin::Holla, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "wait:▎ I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox"]),




    Root::new("holla/flows/docker/done", Bin::Holla, &["--scenario", "docker-cleanup", "--motion", "reduced"], "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "wait:Plan changed", "right", "enter", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "wait:7 succeeded"]).timeout(60_000),


    Root::new("holla/flows/disk/top_files", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "t", "wait:Top files · Spotlight"]).timeout(15_000),









    Root::new("holla/flows/cleanup/report", Bin::Holla, &["--scenario", "parity-cleanup-results", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:TRASH 1 UNDER /Users/alex/Projects/safe ON mbp", "enter", "wait:Cleanup report"]).timeout(15_000),






    Root::new("holla/flows/child/query", Bin::Holla, &["--scenario", "monorepo-child", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:test", "wait:Results · 17"]),



    Root::new("holla/flows/query/empty_preview", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments", "enter", "wait:gate 1 of 2", "escape", "escape", "type:blocking", "enter", "wait:Who is blocking", "escape", "type:resources", "wait:No matches for"]),

    Root::new("holla/flows/remote/query", Bin::Holla, &["--scenario", "remote-host", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:restart payments", "wait:▎ restart payments"]),


    Root::new("holla/flows/args", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:port", "enter", "wait:Find the process on a port", "ctrl-s", "wait:Port 5173"]),


    Root::new("holla/flows/upgrade/facts", Bin::Holla, &["--scenario", "upgrade-plan", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["p", "wait:Back to outline"]),

    Root::new("holla/flows/config/page", Bin::Holla, &["--scenario", "parity-discovery", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Custom action configuration", "enter", "wait:sha256:"]),





    Root::new("holla/flows/files/actions", Bin::Holla, &["--scenario", "parity-files", "--motion", "reduced"], "holla❯").sends(&["type:Find files under home", "enter", "wait:of 29 indexed", "ctrl-u", "type:todo", "wait:1 result", "enter", "wait:Open in the OS"]),

    Root::new("holla/flows/files/jumped", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "g", "wait:type a path or a name", "type:~/work", "enter", "wait:Jumped to ~/work"]),



    Root::new("holla/flows/git/pull_blocked", Bin::Holla, &["--scenario", "parity-git-current", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Pull", "wait:blocked · 1 modified"]),
    Root::new("holla/flows/git/merge", Bin::Holla, &["--scenario", "parity-git-current", "--motion", "reduced"], "holla❯").sends(&["type:Pull with merge", "enter", "right", "enter", "wait:error: Your local changes"]),
    Root::new("holla/flows/git/push_rejected", Bin::Holla, &["--scenario", "parity-git-current", "--motion", "reduced"], "holla❯").sends(&["type:Pull with merge", "enter", "right", "enter", "wait:error: Your local changes", "alt-0", "ctrl-u", "type:Push", "enter", "right", "enter", "wait:rejected"]),


    Root::new("holla/flows/git/batch", Bin::Holla, &["--scenario", "parity-git-batch", "--motion", "reduced"], "holla❯").sends(&["type:Pull 4 sibling", "enter", "right", "enter", "wait:Fast-forward"]).timeout(15_000),

    Root::new("holla/flows/task/sources", Bin::Holla, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:yarn", "wait:yarn s00"]),
    Root::new("holla/flows/task/sources_diagnostic", Bin::Holla, &["--scenario", "parity-task-sources", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Taskfile", "wait:discovery diagnostic"]),

    Root::new("holla/flows/cargo/clean", Bin::Holla, &["--scenario", "parity-cargo", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:cargo clean", "wait:Results · 3"]),







    Root::new("holla/flows/docker/remove_failed", Bin::Holla, &["--scenario", "parity-docker", "--motion", "reduced"], "holla❯").sends(&["type:Stop all containers", "enter", "right", "enter", "wait:exit 1"]),
    Root::new("holla/flows/docker/remove_gate1", Bin::Holla, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Stop and remove all", "enter", "wait:gate 1 of 2"]),
    Root::new("holla/flows/docker/remove_gate2", Bin::Holla, &["--scenario", "parity-docker", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Stop and remove all", "enter", "wait:gate 1 of 2", "right", "enter", "wait:Type REMOVE ALL CONTAINERS ON mbp"]),

    Root::new("holla/flows/brew/services", Bin::Holla, &["--scenario", "parity-brew-services", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Homebrew services", "enter", "wait:svc02"]),
    Root::new("holla/flows/brew/services_stop_failed", Bin::Holla, &["--scenario", "parity-brew-services", "--motion", "reduced"], "holla❯").sends(&["type:Stop svc02", "enter", "right", "enter", "wait:exit 1"]),


    Root::new("holla/flows/gradle/cleanup", Bin::Holla, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Clean Gradle outputs", "enter", "wait:Cleanup · Gradle outputs", "right", "sleep:300"]),
    Root::new("holla/flows/gradle/gate1", Bin::Holla, &["--scenario", "parity-gradle", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Clean Gradle outputs", "enter", "wait:Cleanup · Gradle outputs", "right", "d", "sleep:300"]),
    Root::new("holla/flows/idea/cleanup", Bin::Holla, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Clean IntelliJ", "enter", "right", "wait:idea.clean"]),
    Root::new("holla/flows/idea/gate2", Bin::Holla, &["--scenario", "parity-idea", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Clean IntelliJ", "enter", "right", "wait:idea.clean", "d", "wait:gate 1 of 2", "right", "enter", "wait:Type TRASH 4 UNDER /Users/alex/work/ide ON mbp"]),


    Root::new("holla/flows/brew/upgrade_batch", Bin::Holla, &["--scenario", "parity-upgrade-managers", "--motion", "reduced"], "holla❯").sends(&["type:Upgrade Homebrew packages", "enter", "right", "enter", "wait:Outdated Formulae · 2"]).timeout(15_000),
    Root::new("holla/flows/brew/upgrade_plan", Bin::Holla, &["--scenario", "parity-upgrade-managers", "--motion", "reduced"], "holla❯").sends(&["type:Upgrade Homebrew packages", "enter", "right", "enter", "wait:Outdated Formulae · 2", "alt-0", "ctrl-u", "type:Upgrade everything", "enter", "wait:9 steps"]).timeout(15_000),






    Root::new("holla/flows/task/input_cancelled", Bin::Holla, &["--scenario", "parity-task-input", "--motion", "reduced"], "holla❯").sends(&["type:Deploy the release", "enter", "right", "enter", "wait:Password:", "i", "type:hunter2", "enter", "wait:Proceed with rollout?", "type:n", "enter", "wait:rollout cancelled by operator"]).timeout(15_000),
    Root::new("holla/flows/task/input_killed", Bin::Holla, &["--scenario", "parity-task-input", "--motion", "reduced"], "holla❯").sends(&["type:Deploy the release", "enter", "right", "enter", "wait:Password:", "i", "type:hunter2", "enter", "wait:Proceed with rollout?", "type:n", "enter", "wait:rollout cancelled by operator", "alt-0", "ctrl-u", "type:Run the stubborn worker", "enter", "wait:ignoring SIGTERM", "s", "wait:killed (SIGKILL)"]).timeout(15_000),






    Root::new("holla/flows/trust/deploy_failed", Bin::Holla, &["--scenario", "parity-custom-actions", "--motion", "reduced"], "holla❯").sends(&["type:Deploy preview", "enter", "wait:Trust ~/work/team/.holla.toml?", "right", "enter", "wait:exit 127"]),





    Root::new("holla/flows/disk/scan_complete", Bin::Holla, &["--scenario", "parity-disk-scan", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:2 unreadable"]).timeout(15_000),


    Root::new("holla/flows/disk/tree", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete"]).timeout(15_000),
    Root::new("holla/flows/disk/tree_apparent", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "wait:largest first · apparent"]).timeout(15_000),
    Root::new("holla/flows/disk/tree_unfolded", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "s", "down", "down", "f", "wait:allocated · unfolded"]).timeout(15_000),
    Root::new("holla/flows/disk/tree_selected", Bin::Holla, &["--scenario", "parity-disk-navigation", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:scan complete", "s", "s", "down", "down", "f", "f", "space", "wait:1 selected (176.0"]).timeout(15_000),


    Root::new("holla/flows/cleanup/categories", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:Cleanup review"]),
    Root::new("holla/flows/cleanup/derived_data", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:Cleanup review", "right", "wait:▾ Xcode DerivedData"]),
    Root::new("holla/flows/cleanup/artifacts", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:Cleanup review", "end", "wait:older than 7 days"]).compact(100, &["type:Review cleanup candidates", "enter", "wait:Cleanup review", "end", "wait:Project artifacts"]),



    Root::new("holla/flows/cleanup/gate1", Bin::Holla, &["--scenario", "parity-delete-safety", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2"]).timeout(15_000),
    Root::new("holla/flows/cleanup/gate2", Bin::Holla, &["--scenario", "parity-delete-safety", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2"]).timeout(15_000),
    Root::new("holla/flows/cleanup/gate2_typed", Bin::Holla, &["--scenario", "parity-delete-safety", "--motion", "reduced"], "holla❯").sends(&["type:Analyze disk usage", "enter", "wait:1 unreadable", "down", "space", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:TRASH 1 UNDER /Users/alex/Projects/safe ON mbp", "wait:▎ TRASH 1 UNDER /Users/alex/Projects/safe ON mbp"]).timeout(15_000),

    Root::new("holla/flows/cleanup/history", Bin::Holla, &["--scenario", "parity-cleanup-results", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Show cleanup history", "enter", "wait:operation log records"]),


    Root::new("holla/flows/platforms/linux_gate1", Bin::Holla, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend"]),
    Root::new("holla/flows/platforms/linux_gate2", Bin::Holla, &["--scenario", "parity-platforms-linux", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend", "right", "enter", "wait:Type TRASH 1 UNDER /home/alex ON devbox"]),


    Root::new("holla/flows/cleanup/linux_report", Bin::Holla, &["--scenario", "parity-platforms-linux", "--motion", "reduced"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "right", "down", "space", "d", "wait:no Trash backend", "right", "enter", "wait:Type TRASH 1 UNDER /home/alex ON devbox", "enter", "type:TRASH 1 UNDER /home/alex ON devbox", "enter", "wait:Cleanup report"]).timeout(15_000),
    Root::new("holla/flows/platforms/files", Bin::Holla, &["--scenario", "parity-platforms", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Top files on this Mac", "enter", "wait:did not finish within 5 s"]),



    Root::new("holla/flows/docker/drift", Bin::Holla, &["--scenario", "docker-cleanup", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["enter", "c", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2", "enter", "type:I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox", "enter", "wait:a new container"]),



    Root::new("holla/fade/browser_wheel", Bin::Holla, &["--scenario", "parity-browser", "--motion", "reduced"], "holla❯").sends(&["type:Browse ~/work/site", "enter", "wait:16 entries", "down", "down", "down", "wait:first 2000 lines"]).compact(100, &["type:Browse ~/work/site", "enter", "wait:16 entries", "home", "down", "down", "down", "right", "wait:first 2000 lines"]).live(interact::browser_wheel),
    Root::new("holla/fade/trust_body_wheel", Bin::Holla, &["--scenario", "parity-custom-actions", "--motion", "reduced"], "holla❯").sends(&["type:Deploy preview", "enter", "wait:Trust ~/work/team/.holla.toml?"]).live(interact::trust_body_wheel),



    Root::new("holla/fade/cleanup_list_wheel", Bin::Holla, &["--scenario", "parity-insights", "--motion", "paused", "--frame", "40"], "holla❯").sends(&["type:Review cleanup candidates", "enter", "wait:Cleanup review"]).live(interact::cleanup_list_wheel),

    Root::new("holla/fade/executor_burst-end", Bin::Holla, &["--scenario", "parity-executor", "--motion", "reduced"], "holla❯").sends(&["type:Emit a burst", "enter", "wait:500 earlier lines dropped"]),
];

#[rustfmt::skip]
pub const RESIZES: &[Root] = &[
    Root::new("holla/resize/rust-dirty_shrunk", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").timeout(30_000).resize((120, 40), interact::resize_case),
    Root::new("holla/resize/rust-dirty_grown", Bin::Holla, &["--scenario", "rust-dirty", "--motion", "paused", "--frame", "40"], "holla❯").timeout(30_000).resize((80, 24), interact::resize_case),

    Root::new("holla/resize/first-use_from160x50", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").timeout(30_000).resize((160, 50), interact::resize_case).pending(),
    Root::new("holla/resize/first-use_from72x20", Bin::Holla, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], "holla❯").timeout(30_000).resize((72, 20), interact::resize_case).pending(),
];
