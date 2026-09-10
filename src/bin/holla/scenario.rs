//! Deterministic scenario contract: which fixture world Holla opens in,
//! how motion behaves, and which tick a paused capture shows.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    /// An empty scratch folder on a quiet laptop: the root must still be
    /// useful without a project.
    FirstUse,
    /// A Rust project with a dirty worktree, a branch behind upstream and a
    /// large target directory.
    RustDirty,
    /// A mise monorepo root with bounded child projects and a Compose stack.
    MonorepoRoot,
    /// Inside `apps/frontend` of the same monorepo: parent ecosystem actions
    /// and an untrusted child configuration.
    MonorepoChild,
    /// A Docker host with containers, images and volumes; the query
    /// `docker clean` is already typed.
    DockerCleanup,
    /// A project collection under `~/work` with the Disk flow open and the
    /// scan still streaming.
    DiskCleanup,
    /// A Debian host with the "Upgrade everything" plan open for review.
    UpgradePlan,
    /// Several activities running, finished and detached at once.
    ActivitiesMulti,
    /// A production host reached over SSH.
    RemoteHost,
    /// A child task whose launch fails.
    LaunchFailure,
    /// Long labels, an unreachable Docker daemon, partial discovery, many
    /// rows.
    HardCases,
    // ---- parity fixtures (HP01–HP23): one world per legacy capability family
    ParityDiscovery,
    ParityHistory,
    ParityFiles,
    ParityBrowser,
    ParityGitCurrent,
    ParityGitBatch,
    ParityTaskSources,
    ParityCargo,
    ParityDocker,
    ParityBrewServices,
    ParityGradle,
    ParityIdea,
    ParityUpgradeManagers,
    ParityExecutor,
    ParityTaskInput,
    ParityCustomActions,
    ParityDiskScan,
    ParityDiskNavigation,
    ParityInsights,
    ParityDeleteSafety,
    ParityCleanupResults,
    ParityPlatforms,
    ParityPlatformsLinux,
}

impl Scenario {
    pub const CONCEPT: [Scenario; 11] = [
        Scenario::FirstUse,
        Scenario::RustDirty,
        Scenario::MonorepoRoot,
        Scenario::MonorepoChild,
        Scenario::DockerCleanup,
        Scenario::DiskCleanup,
        Scenario::UpgradePlan,
        Scenario::ActivitiesMulti,
        Scenario::RemoteHost,
        Scenario::LaunchFailure,
        Scenario::HardCases,
    ];

    pub const PARITY: [Scenario; 23] = [
        Scenario::ParityDiscovery,
        Scenario::ParityHistory,
        Scenario::ParityFiles,
        Scenario::ParityBrowser,
        Scenario::ParityGitCurrent,
        Scenario::ParityGitBatch,
        Scenario::ParityTaskSources,
        Scenario::ParityCargo,
        Scenario::ParityDocker,
        Scenario::ParityBrewServices,
        Scenario::ParityGradle,
        Scenario::ParityIdea,
        Scenario::ParityUpgradeManagers,
        Scenario::ParityExecutor,
        Scenario::ParityTaskInput,
        Scenario::ParityCustomActions,
        Scenario::ParityDiskScan,
        Scenario::ParityDiskNavigation,
        Scenario::ParityInsights,
        Scenario::ParityDeleteSafety,
        Scenario::ParityCleanupResults,
        Scenario::ParityPlatforms,
        Scenario::ParityPlatformsLinux,
    ];

    pub const ALL: [Scenario; 34] = [
        Scenario::FirstUse,
        Scenario::RustDirty,
        Scenario::MonorepoRoot,
        Scenario::MonorepoChild,
        Scenario::DockerCleanup,
        Scenario::DiskCleanup,
        Scenario::UpgradePlan,
        Scenario::ActivitiesMulti,
        Scenario::RemoteHost,
        Scenario::LaunchFailure,
        Scenario::HardCases,
        Scenario::ParityDiscovery,
        Scenario::ParityHistory,
        Scenario::ParityFiles,
        Scenario::ParityBrowser,
        Scenario::ParityGitCurrent,
        Scenario::ParityGitBatch,
        Scenario::ParityTaskSources,
        Scenario::ParityCargo,
        Scenario::ParityDocker,
        Scenario::ParityBrewServices,
        Scenario::ParityGradle,
        Scenario::ParityIdea,
        Scenario::ParityUpgradeManagers,
        Scenario::ParityExecutor,
        Scenario::ParityTaskInput,
        Scenario::ParityCustomActions,
        Scenario::ParityDiskScan,
        Scenario::ParityDiskNavigation,
        Scenario::ParityInsights,
        Scenario::ParityDeleteSafety,
        Scenario::ParityCleanupResults,
        Scenario::ParityPlatforms,
        Scenario::ParityPlatformsLinux,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Scenario::FirstUse => "first-use",
            Scenario::RustDirty => "rust-dirty",
            Scenario::MonorepoRoot => "monorepo-root",
            Scenario::MonorepoChild => "monorepo-child",
            Scenario::DockerCleanup => "docker-cleanup",
            Scenario::DiskCleanup => "disk-cleanup",
            Scenario::UpgradePlan => "upgrade-plan",
            Scenario::ActivitiesMulti => "activities-multi",
            Scenario::RemoteHost => "remote-host",
            Scenario::LaunchFailure => "launch-failure",
            Scenario::HardCases => "hard-cases",
            Scenario::ParityDiscovery => "parity-discovery",
            Scenario::ParityHistory => "parity-history",
            Scenario::ParityFiles => "parity-files",
            Scenario::ParityBrowser => "parity-browser",
            Scenario::ParityGitCurrent => "parity-git-current",
            Scenario::ParityGitBatch => "parity-git-batch",
            Scenario::ParityTaskSources => "parity-task-sources",
            Scenario::ParityCargo => "parity-cargo",
            Scenario::ParityDocker => "parity-docker",
            Scenario::ParityBrewServices => "parity-brew-services",
            Scenario::ParityGradle => "parity-gradle",
            Scenario::ParityIdea => "parity-idea",
            Scenario::ParityUpgradeManagers => "parity-upgrade-managers",
            Scenario::ParityExecutor => "parity-executor",
            Scenario::ParityTaskInput => "parity-task-input",
            Scenario::ParityCustomActions => "parity-custom-actions",
            Scenario::ParityDiskScan => "parity-disk-scan",
            Scenario::ParityDiskNavigation => "parity-disk-navigation",
            Scenario::ParityInsights => "parity-insights",
            Scenario::ParityDeleteSafety => "parity-delete-safety",
            Scenario::ParityCleanupResults => "parity-cleanup-results",
            Scenario::ParityPlatforms => "parity-platforms",
            Scenario::ParityPlatformsLinux => "parity-platforms-linux",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|sc| sc.name() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    /// Ticks drive discovery, activities and plans.
    #[default]
    Full,
    /// Discovery completes at once; activities and plans still advance.
    Reduced,
    /// Ticks never advance; `--frame` selects the exact frame shown.
    Paused,
}

impl Motion {
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Motion::Full),
            "reduced" => Some(Motion::Reduced),
            "paused" => Some(Motion::Paused),
            _ => None,
        }
    }

    /// Explicit CLI motion wins; otherwise `HOLLA_NO_MOTION` selects the
    /// reduced path.
    pub fn resolve(cli: Option<Motion>, no_motion_env: bool) -> Motion {
        match cli {
            Some(m) => m,
            None if no_motion_env => Motion::Reduced,
            None => Motion::Full,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for s in Scenario::ALL {
            assert_eq!(Scenario::from_name(s.name()), Some(s));
        }
        assert_eq!(Motion::resolve(None, true), Motion::Reduced);
        assert_eq!(Motion::resolve(Some(Motion::Full), true), Motion::Full);
        assert_eq!(Motion::resolve(None, false), Motion::Full);
    }
}
