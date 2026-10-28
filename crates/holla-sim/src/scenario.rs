#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    FirstUse,
    RustDirty,
    MonorepoRoot,
    MonorepoChild,
    DockerCleanup,
    DiskCleanup,
    UpgradePlan,
    ActivitiesMulti,
    RemoteHost,
    LaunchFailure,
    HardCases,
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
    ParityActivityStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Finder,
    Disk,
    Plan,
    Activity,
    Files,
    Cleanup,
    Review,
    Snapshot,
}

impl Screen {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Finder => "finder",
            Self::Disk => "disk",
            Self::Plan => "plan",
            Self::Activity => "activity",
            Self::Files => "files",
            Self::Cleanup => "cleanup",
            Self::Review => "review",
            Self::Snapshot => "snapshot",
        }
    }
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

    pub const PARITY: [Scenario; 24] = [
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
        Scenario::ParityActivityStop,
    ];

    pub const ALL: [Scenario; 35] = [
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
        Scenario::ParityActivityStop,
    ];

    pub const fn name(self) -> &'static str {
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
            Scenario::ParityActivityStop => "parity-activity-stop",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|sc| sc.name() == s)
    }

    pub const fn screen(self) -> Screen {
        match self {
            Scenario::DiskCleanup => Screen::Disk,
            Scenario::UpgradePlan => Screen::Plan,
            Scenario::LaunchFailure => Screen::Activity,
            Scenario::ParityActivityStop => Screen::Activity,
            Scenario::FirstUse
            | Scenario::RustDirty
            | Scenario::MonorepoRoot
            | Scenario::MonorepoChild
            | Scenario::DockerCleanup
            | Scenario::ActivitiesMulti
            | Scenario::RemoteHost
            | Scenario::HardCases
            | Scenario::ParityDiscovery
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
            | Scenario::ParityPlatformsLinux => Screen::Finder,
        }
    }

    pub const fn boot_title(self) -> &'static str {
        match self {
            Scenario::DiskCleanup => "Disk usage",
            Scenario::LaunchFailure => "failed",
            Scenario::ParityActivityStop => "ordinary worker",
            Scenario::ParityFiles
            | Scenario::ParityDiskScan
            | Scenario::ParityPlatforms
            | Scenario::ParityPlatformsLinux
            | Scenario::ParityBrewServices
            | Scenario::ParityUpgradeManagers => "Recent here",
            _ => "Suggested here",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    #[default]
    Full,
    Reduced,
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

    pub fn resolve(cli: Option<Motion>, no_motion_env: bool) -> Motion {
        match cli {
            Some(m) => m,
            None if no_motion_env => Motion::Reduced,
            None => Motion::Full,
        }
    }
}
