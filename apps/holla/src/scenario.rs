//! Deterministic scenario contract: which fixture world the launcher starts
//! in, how motion behaves, and which tick a paused capture shows.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    /// Empty directory on a local dev host: nothing discovered yet.
    FirstUse,
    /// A Rust project with a dirty worktree and a branch behind upstream.
    RustDirty,
    /// Monorepository root with bounded child projects and mise tasks.
    MonorepoRoot,
    /// Inside `apps/frontend` of the monorepo; parent ecosystem visible.
    MonorepoChild,
    /// Docker host where the complete-cleanup journey is the point.
    DockerCleanup,
    /// Disk pressure; progressive analysis and cleanup candidates.
    DiskCleanup,
    /// Debian host with global mise tools; the upgrade-everything plan.
    UpgradePlan,
    /// Several named activities running at once (dev server, logs, tests).
    ActivitiesMulti,
    /// SSH into a sensitive remote host; stronger identity and gates.
    RemoteHost,
    /// An action whose simulated execution fails.
    LaunchFailure,
    /// Long labels, missing data, discovery failure, narrow width.
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
    /// Tick-driven spinners and progressive discovery.
    #[default]
    Full,
    /// Static as soon as the fixture settles; ticks still advance time.
    Reduced,
    /// Ticks never advance; `--frame` selects the exact fixture tick shown.
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
        assert_eq!(Scenario::ALL.len(), 11);
        assert_eq!(Motion::resolve(None, true), Motion::Reduced);
        assert_eq!(Motion::resolve(Some(Motion::Full), true), Motion::Full);
        assert_eq!(Motion::resolve(None, false), Motion::Full);
    }
}
