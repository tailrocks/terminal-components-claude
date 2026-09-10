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
}

impl Scenario {
    pub const ALL: [Scenario; 11] = [
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
